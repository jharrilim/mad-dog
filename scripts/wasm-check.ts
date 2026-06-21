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
  run_refinement_quench_json,
  run_refinement_n_compare_json,
  run_relational_time_json,
  run_universe_3d_json,
  run_universe_slice_json,
  run_factorization_search_json,
} from '../src/sim/wasm/pkg/mad_dog_sim.js'
import {
  runEmergence,
  runHolography,
  runRtMass,
  runSpacetime,
  runRefinementQuench,
  runRefinementNCompare,
  runRelationalTime,
  runUniverse3D,
  runUniverseSlice,
  runFactorizationSearch,
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

// --- Refinement quench ---
{
  const config = { n: 10, field: 1.5, dt: 0.2, steps: 16, seed: 7711 }
  const ts = runRefinementQuench(config)
  const wasm = JSON.parse(
    run_refinement_quench_json(JSON.stringify(config)),
  ) as typeof ts
  const tsLate = ts.slices[ts.slices.length - 1].diagnostics
  const wasmLate = wasm.slices[wasm.slices.length - 1].diagnostics
  console.log('\nrefinement quench (10-site):')
  console.log('  late pressure', approx(tsLate.pressure, wasmLate.pressure, 1e-4))
  console.log(
    '  needs refine  ',
    tsLate.needsRefinement === wasmLate.needsRefinement ? 'OK' : 'MISMATCH',
  )
  fail(
    Math.abs(tsLate.pressure - wasmLate.pressure) >= 1e-4 ||
      tsLate.needsRefinement !== wasmLate.needsRefinement,
  )
}

// --- Refinement n compare ---
{
  const config = {
    n: 10,
    deltaN: 2,
    field: 1.5,
    dt: 0.2,
    quenchStep: 12,
    seed: 7711,
  }
  const ts = runRefinementNCompare(config)
  const wasm = JSON.parse(
    run_refinement_n_compare_json(JSON.stringify(config)),
  ) as typeof ts
  console.log('\nrefinement n-compare:')
  console.log('  small pressure', approx(ts.small.pressure, wasm.small.pressure, 1e-4))
  console.log('  large pressure', approx(ts.large.pressure, wasm.large.pressure, 1e-4))
  console.log(
    '  larger relieves',
    ts.largerRelieves === wasm.largerRelieves ? 'OK' : 'MISMATCH',
  )
  fail(
    Math.abs(ts.small.pressure - wasm.small.pressure) >= 1e-4 ||
      Math.abs(ts.large.pressure - wasm.large.pressure) >= 1e-4 ||
      ts.largerRelieves !== wasm.largerRelieves,
  )
}

// --- Relational time ---
{
  const config = {
    n: 9,
    field: 1,
    dt: 0.2,
    steps: 40,
    clockSite: 0,
    physicalSlices: 15,
  }
  const ts = runRelationalTime(config)
  const wasm = JSON.parse(
    run_relational_time_json(JSON.stringify(config)),
  ) as typeof ts
  console.log('\nrelational time (9-site, edge clock):')
  console.log('  sync R²   ', approx(ts.syncR2, wasm.syncR2, 1e-4))
  console.log('  sync slope', approx(ts.syncSlope, wasm.syncSlope, 1e-4))
  fail(
    Math.abs(ts.syncR2 - wasm.syncR2) >= 1e-4 ||
      Math.abs(ts.syncSlope - wasm.syncSlope) >= 1e-4,
  )
}

// --- Universe 3D ---
{
  const config = { lx: 2, ly: 2, lz: 2, field: 1, dt: 0.25, steps: 12 }
  const ts = runUniverse3D(config)
  const wasm = JSON.parse(run_universe_3d_json(JSON.stringify(config))) as typeof ts
  console.log('\nuniverse 3d (2x2x2):')
  console.log(
    '  LR velocity',
    approx(ts.lightCone.velocity, wasm.lightCone.velocity, 1e-3),
  )
  console.log(
    '  energy drift',
    approx(ts.spacetime.energyDrift, wasm.spacetime.energyDrift, 1e-4),
  )
  const dim3 = wasm.spacetime.slices[0].coords.every((c) => c.length >= 3)
  console.log('  3D coords   ', dim3 ? 'OK' : 'NO')
  fail(
    Math.abs(ts.lightCone.velocity - wasm.lightCone.velocity) >= 1e-3 ||
      Math.abs(ts.spacetime.energyDrift - wasm.spacetime.energyDrift) >= 1e-4 ||
      !dim3,
  )
}

// --- Universe slice ---
{
  const config = { lx: 2, ly: 2, lz: 2, field: 1, dt: 0.25, steps: 12, k: 3 }
  const ts = runUniverseSlice(config)
  const wasm = JSON.parse(
    run_universe_slice_json(JSON.stringify(config)),
  ) as typeof ts
  console.log('\nuniverse slice (k=3):')
  console.log(
    '  emergent dim',
    ts.report.mds.emergentDim === wasm.report.mds.emergentDim ? 'OK' : 'MISMATCH',
  )
  fail(ts.report.mds.emergentDim !== wasm.report.mds.emergentDim)
}

// --- Factorization search ---
{
  const config = {
    kind: 'shuffled_chain',
    n: 6,
    field: 1.5,
    seed: 4242,
    topK: 3,
  }
  const ts = runFactorizationSearch(config)
  const wasm = JSON.parse(
    run_factorization_search_json(JSON.stringify(config)),
  ) as typeof ts
  console.log('\nfactorization (shuffled 6-chain):')
  console.log('  best score  ', approx(ts.best.score, wasm.best.score, 1e-4))
  console.log(
    '  recovered   ',
    ts.recoveredIdentity === wasm.recoveredIdentity ? 'OK' : 'MISMATCH',
  )
  console.log(
    '  best locality',
    approx(ts.best.localityFraction, wasm.best.localityFraction, 1e-4),
  )
  fail(
    Math.abs(ts.best.score - wasm.best.score) >= 1e-4 ||
      ts.recoveredIdentity !== wasm.recoveredIdentity,
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
