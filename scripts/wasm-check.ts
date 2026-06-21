/**
 * Compare TypeScript reference vs Rust/WASM.
 * Run:  npm run check:wasm
 */

import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import {
  initSync,
  run_emergence_json,
  run_spacetime_json,
  run_holography_json,
  run_rt_mass_json,
} from '../src/sim/wasm/pkg/mad_dog_sim.js'
import {
  runEmergence,
  runHolography,
  runRtMass,
  runSpacetime,
  type RunConfig,
} from '../src/sim/runner.ts'

const __dirname = dirname(fileURLToPath(import.meta.url))
const wasmPath = join(__dirname, '../src/sim/wasm/pkg/mad_dog_sim_bg.wasm')
initSync({ module: readFileSync(wasmPath) })

function approx(a: number, b: number, tol = 1e-6): string {
  return Math.abs(a - b) < tol ? 'OK' : `MISMATCH (got ${a}, want ${b})`
}

let failures = 0

function fail(cond: boolean) {
  if (cond) failures++
}

// --- Emergence ---
const emergenceCases: RunConfig[] = [
  { kind: 'chain', n: 8, rows: 3, cols: 3, field: 1.5, seed: 12345 },
  { kind: 'grid', n: 9, rows: 3, cols: 3, field: 1.5, seed: 12345 },
  { kind: 'random', n: 8, rows: 3, cols: 3, field: 1.5, seed: 777 },
]

for (const config of emergenceCases) {
  const ts = runEmergence(config)
  const wasm = JSON.parse(run_emergence_json(JSON.stringify(config))) as typeof ts

  console.log(`\nemergence/${config.kind} (seed=${config.seed}):`)
  console.log('  energy   ', approx(ts.energy, wasm.energy, 1e-5))
  console.log(
    '  emerg dim',
    ts.report.mds.emergentDim === wasm.report.mds.emergentDim ? 'OK' : 'MISMATCH',
  )
  const miMaxDiff = maxAbsDiff(ts.report.mi, wasm.report.mi)
  console.log('  MI max Δ  ', miMaxDiff < 1e-5 ? 'OK' : `HIGH (${miMaxDiff.toExponential(2)})`)
  fail(
    Math.abs(ts.energy - wasm.energy) >= 1e-5 ||
      ts.report.mds.emergentDim !== wasm.report.mds.emergentDim ||
      miMaxDiff >= 1e-5,
  )
}

// --- Spacetime ---
{
  const config = { n: 9, field: 1, dt: 0.2, steps: 12, seed: 7 }
  const ts = runSpacetime(config)
  const wasm = JSON.parse(run_spacetime_json(JSON.stringify(config))) as typeof ts
  console.log('\nspacetime (9-site chain):')
  console.log('  energy drift', approx(ts.energyDrift, wasm.energyDrift, 1e-4))
  console.log('  slices       ', ts.slices.length === wasm.slices.length ? 'OK' : 'MISMATCH')
  const sigDiff = maxAbsDiff(
    ts.slices.map((s) => s.signal),
    wasm.slices.map((s) => s.signal),
  )
  console.log('  signal max Δ ', sigDiff < 1e-4 ? 'OK' : `HIGH (${sigDiff.toExponential(2)})`)
  fail(ts.energyDrift - wasm.energyDrift >= 1e-4 || sigDiff >= 1e-4)
}

// --- Holography ---
{
  const config = { n: 10, field: 1.5, seed: 31337 }
  const ts = runHolography(config)
  const wasm = JSON.parse(run_holography_json(JSON.stringify(config))) as typeof ts
  console.log('\nholography (10-site):')
  console.log('  RT slope     ', approx(ts.report.rtSlope, wasm.report.rtSlope, 1e-4))
  console.log('  RT R²        ', approx(ts.report.rtR2, wasm.report.rtR2, 1e-4))
  fail(
    Math.abs(ts.report.rtSlope - wasm.report.rtSlope) >= 1e-4 ||
      Math.abs(ts.report.rtR2 - wasm.report.rtR2) >= 1e-4,
  )
}

// --- RT mass ---
{
  const config = { n: 10, field: 1.5, seed: 7, strength: 1.0 }
  const ts = runRtMass(config)
  const wasm = JSON.parse(run_rt_mass_json(JSON.stringify(config))) as typeof ts
  console.log('\nRT mass (10-site):')
  console.log(
    '  vacuum slope ',
    approx(ts.report.vacuum.rtSlope, wasm.report.vacuum.rtSlope, 1e-4),
  )
  console.log(
    '  mass slope   ',
    approx(ts.report.mass.rtSlope, wasm.report.mass.rtSlope, 1e-4),
  )
  fail(
    Math.abs(ts.report.vacuum.rtSlope - wasm.report.vacuum.rtSlope) >= 1e-4 ||
      Math.abs(ts.report.mass.rtSlope - wasm.report.mass.rtSlope) >= 1e-4,
  )
}

console.log(failures === 0 ? '\nAll WASM checks passed.' : `\n${failures} case(s) failed.`)
process.exit(failures === 0 ? 0 : 1)

function maxAbsDiff(a: number[][], b: number[][]): number {
  let m = 0
  for (let i = 0; i < a.length; i++) {
    for (let j = 0; j < a[i].length; j++) {
      m = Math.max(m, Math.abs(a[i][j] - b[i][j]))
    }
  }
  return m
}
