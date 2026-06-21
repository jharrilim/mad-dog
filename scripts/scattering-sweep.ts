/**
 * Parallel scattering parameter sweep (WASM-only, lite mode).
 * Run: npm run sweep:scattering
 *
 * CLI: node scripts/scattering-sweep.ts [n...] [--h 0.6,0.7,0.8] [--steps 40] [--workers 4]
 */

import { Worker } from 'node:worker_threads'
import { cpus } from 'node:os'
import { fileURLToPath } from 'node:url'

interface SweepCase {
  n: number
  field: number
  dt: number
  steps: number
  defectSites: number[]
}

interface SweepRow {
  n: number
  field: number
  crossed: boolean
  minSep: number
  elapsedMs: number
}

function parseArgs(argv: string[]) {
  let ns = [12, 16, 20]
  let hs = [0.6, 0.7, 0.8]
  let steps = 40
  let workers = Math.min(4, Math.max(1, cpus().length))

  const positional: number[] = []
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i]
    if (a === '--h' && argv[i + 1]) {
      hs = argv[++i].split(',').map(Number)
    } else if (a === '--steps' && argv[i + 1]) {
      steps = Number(argv[++i])
    } else if (a === '--workers' && argv[i + 1]) {
      workers = Number(argv[++i])
    } else if (/^\d+$/.test(a)) {
      positional.push(Number(a))
    }
  }
  if (positional.length) ns = positional

  return { ns, hs, steps, workers }
}

function defaultDefects(n: number): number[] {
  return [Math.floor(n / 4), Math.floor((3 * n) / 4)]
}

function runCase(config: SweepCase): Promise<SweepRow> {
  return new Promise((resolve, reject) => {
    const worker = new Worker(new URL('./scattering-sweep-worker.ts', import.meta.url), {
      workerData: config,
    })
    worker.on('message', resolve)
    worker.on('error', reject)
    worker.on('exit', (code) => {
      if (code !== 0) reject(new Error(`worker exited ${code}`))
    })
  })
}

async function pool<T, R>(items: T[], concurrency: number, fn: (item: T) => Promise<R>): Promise<R[]> {
  const out: R[] = []
  let i = 0
  async function worker() {
    while (i < items.length) {
      const idx = i++
      out[idx] = await fn(items[idx])
    }
  }
  await Promise.all(Array.from({ length: Math.min(concurrency, items.length) }, () => worker()))
  return out
}

const { ns, hs, steps, workers } = parseArgs(process.argv.slice(2))
const cases: SweepCase[] = []
for (const n of ns) {
  for (const field of hs) {
    cases.push({ n, field, dt: 0.12, steps, defectSites: defaultDefects(n) })
  }
}

const maxCost = cases.reduce((s, c) => s + c.n * c.steps, 0)
if (maxCost > 5000) {
  console.warn(
    `[scattering-sweep] Large grid (${cases.length} cases, Σ n×steps≈${maxCost}); consider --workers or smaller n/steps.`,
  )
}

console.log(`Scattering sweep: ${cases.length} cases, ${workers} workers, lite=true, steps=${steps}`)
const t0 = performance.now()
const rows = await pool(cases, workers, runCase)
const totalMs = performance.now() - t0

console.log('\n  n    h   crossed  minSep   ms')
console.log('  ---  --- -------- ------ -----')
for (const r of rows) {
  console.log(
    `  ${String(r.n).padStart(3)}  ${r.field.toFixed(1)}  ${r.crossed ? 'yes     ' : 'no      '} ${r.minSep.toFixed(3).padStart(6)} ${r.elapsedMs.toFixed(0).padStart(5)}`,
  )
}
console.log(`\nTotal wall time: ${totalMs.toFixed(0)} ms`)
