/**
 * Universe Lab dimension benchmarks (WASM-only).
 * Run: npm run bench:universe
 */

import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import {
  initSync,
  run_emergence_json,
  run_universe_slice_json,
} from '../src/sim/wasm/pkg/mad_dog_sim.js'

const __dirname = dirname(fileURLToPath(import.meta.url))
initSync({ module: readFileSync(join(__dirname, '../src/sim/wasm/pkg/mad_dog_sim_bg.wasm')) })

type Case = {
  label: string
  run: () => { emergentDim: number; expected: number; eigenvalues: number[] }
}

const cases: Case[] = [
  {
    label: 'ground 2×2×2',
    run: () => {
      const r = JSON.parse(
        run_emergence_json(
          JSON.stringify({ kind: 'cube', lx: 2, ly: 2, lz: 2, field: 1.5, seed: 42 }),
        ),
      )
      return {
        emergentDim: r.report.mds.emergentDim,
        expected: 1,
        eigenvalues: r.report.mds.eigenvalues,
      }
    },
  },
  {
    label: 'ground 2×2×3',
    run: () => {
      const r = JSON.parse(
        run_emergence_json(
          JSON.stringify({ kind: 'cube', lx: 2, ly: 2, lz: 3, field: 1.5, seed: 42 }),
        ),
      )
      return {
        emergentDim: r.report.mds.emergentDim,
        expected: r.expectedDim,
        eigenvalues: r.report.mds.eigenvalues,
      }
    },
  },
  {
    label: 'quench 2×2×2 @ k=3',
    run: () => {
      const r = JSON.parse(
        run_universe_slice_json(
          JSON.stringify({ lx: 2, ly: 2, lz: 2, field: 1.5, dt: 0.25, steps: 12, k: 3 }),
        ),
      )
      return {
        emergentDim: r.report.mds.emergentDim,
        expected: 3,
        eigenvalues: r.report.mds.eigenvalues,
      }
    },
  },
  {
    label: 'quench 2×2×3 @ k=3',
    run: () => {
      const r = JSON.parse(
        run_universe_slice_json(
          JSON.stringify({ lx: 2, ly: 2, lz: 3, field: 1.5, dt: 0.25, steps: 12, k: 3 }),
        ),
      )
      return {
        emergentDim: r.report.mds.emergentDim,
        expected: 3,
        eigenvalues: r.report.mds.eigenvalues,
      }
    },
  },
]

let failures = 0

console.warn('[mad-dog universe] bench compares emergent dim vs lattice expectation (h=1.5).')

for (const { label, run } of cases) {
  const t0 = performance.now()
  const { emergentDim, expected, eigenvalues } = run()
  const elapsedMs = performance.now() - t0
  const top = eigenvalues
    .filter((v: number) => v > 1e-6)
    .slice(0, 4)
    .map((v: number) => v.toFixed(3))
    .join(', ')
  const ok = emergentDim >= expected
  if (!ok) failures++
  console.log(
    `\n${label} (${elapsedMs.toFixed(0)} ms): dim=${emergentDim} expected≥${expected}${ok ? ' OK' : ' FAIL'}`,
  )
  console.log(`  top λ: ${top}`)
}

console.log(failures === 0 ? '\nAll universe benchmarks passed.' : `\n${failures} case(s) failed.`)
process.exit(failures === 0 ? 0 : 1)
