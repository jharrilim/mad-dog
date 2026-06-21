/**
 * WASM smoke + structural invariants (Rust is source of truth).
 * Run: npm run check:wasm
 */

import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import {
  initSync,
  wasm_sim_version,
  run_emergence_json,
  run_spacetime_json,
  run_holography_json,
  run_rt_mass_json,
  run_refinement_quench_json,
  run_refinement_n_compare_json,
  run_relational_time_json,
  run_universe_3d_json,
  run_universe_slice_json,
  run_light_cone_compare_json,
  run_modular_dual_clock_json,
  run_scattering_json,
  run_factorization_search_json,
} from '../src/sim/wasm/pkg/mad_dog_sim.js'

const __dirname = dirname(fileURLToPath(import.meta.url))
initSync({ module: readFileSync(join(__dirname, '../src/sim/wasm/pkg/mad_dog_sim_bg.wasm')) })

let failures = 0
function fail(msg: string) {
  console.log('  FAIL:', msg)
  failures++
}
function ok(msg: string) {
  console.log('  OK:', msg)
}

console.log('WASM version:', wasm_sim_version())

// --- Emergence ---
for (const config of [
  { kind: 'chain', n: 8, rows: 3, cols: 3, field: 1.5, seed: 12345 },
  { kind: 'grid', n: 9, rows: 3, cols: 3, field: 1.5, seed: 12345 },
  { kind: 'random', n: 8, rows: 3, cols: 3, field: 1.5, seed: 777 },
] as const) {
  const r = JSON.parse(run_emergence_json(JSON.stringify(config)))
  console.log(`\nemergence/${config.kind}:`)
  if (!Number.isFinite(r.energy)) fail('energy not finite')
  else ok(`energy=${r.energy.toFixed(4)}`)
  if (r.report.mi.length !== config.n) fail('MI size')
  else ok(`emergentDim=${r.report.mds.emergentDim}`)
}

// --- Spacetime ---
{
  const r = JSON.parse(
    run_spacetime_json(JSON.stringify({ n: 9, field: 1, dt: 0.2, steps: 12, seed: 7 })),
  )
  console.log('\nspacetime:')
  if (r.slices.length !== 12) fail(`expected 12 slices, got ${r.slices.length}`)
  else ok(`${r.slices.length} slices`)
  if (!r.lightCone?.velocity || r.lightCone.velocity <= 0) fail('lightCone velocity')
  else ok(`LR velocity=${r.lightCone.velocity.toFixed(3)}`)
}

// --- Holography ---
{
  const r = JSON.parse(
    run_holography_json(JSON.stringify({ n: 10, field: 1.5, seed: 31337 })),
  )
  console.log('\nholography:')
  if (r.report.rtR2 < 0.85) fail(`RT R²=${r.report.rtR2}`)
  else ok(`RT R²=${r.report.rtR2.toFixed(4)}`)
}

// --- RT mass ---
{
  const r = JSON.parse(
    run_rt_mass_json(JSON.stringify({ n: 10, field: 1.5, seed: 7, strength: 1.0 })),
  )
  console.log('\nRT mass:')
  if (r.report.mass.rtSlope <= r.report.vacuum.rtSlope) fail('mass slope not above vacuum')
  else ok(`Δslope=${(r.report.mass.rtSlope - r.report.vacuum.rtSlope).toFixed(3)}`)
}

// --- Refinement ---
{
  const r = JSON.parse(
    run_refinement_quench_json(
      JSON.stringify({ n: 10, field: 1.5, dt: 0.2, steps: 16, seed: 7711 }),
    ),
  )
  console.log('\nrefinement quench:')
  const late = r.slices[r.slices.length - 1].diagnostics
  ok(`late pressure=${late.pressure.toFixed(3)}, decouplingLag=${r.decouplingLag}`)
}

{
  const r = JSON.parse(
    run_refinement_n_compare_json(
      JSON.stringify({ n: 10, deltaN: 2, field: 1.5, dt: 0.2, quenchStep: 12, seed: 7711 }),
    ),
  )
  console.log('\nrefinement n-compare:')
  ok(`small=${r.small.pressure.toFixed(3)} large=${r.large.pressure.toFixed(3)}`)
}

// --- Relational time ---
{
  const r = JSON.parse(
    run_relational_time_json(
      JSON.stringify({ n: 9, field: 1, dt: 0.2, steps: 40, clockSite: 0, physicalSlices: 15 }),
    ),
  )
  console.log('\nrelational time:')
  ok(`syncR²=${r.syncR2.toFixed(4)}`)
}

// --- Universe ---
{
  const r = JSON.parse(
    run_universe_3d_json(JSON.stringify({ lx: 2, ly: 2, lz: 2, field: 1, dt: 0.25, steps: 12 })),
  )
  console.log('\nuniverse 3d:')
  const dim3 = r.spacetime.slices[0].coords.every((c: number[]) => c.length >= 3)
  if (!dim3) fail('coords not 3D')
  else ok(`LR velocity=${r.lightCone.velocity.toFixed(3)}`)
}

{
  const r = JSON.parse(
    run_universe_slice_json(
      JSON.stringify({ lx: 2, ly: 2, lz: 2, field: 1, dt: 0.25, steps: 12, k: 3 }),
    ),
  )
  console.log('\nuniverse slice:')
  ok(`emergentDim=${r.report.mds.emergentDim}`)
}

// --- Factorization ---
{
  const r = JSON.parse(
    run_factorization_search_json(
      JSON.stringify({ kind: 'shuffled_chain', n: 6, field: 1.5, seed: 4242, topK: 3 }),
    ),
  )
  console.log('\nfactorization (n=6):')
  if (!r.recoveredIdentity) fail('identity not recovered')
  else ok(`score=${r.best.score.toFixed(3)} locality=${(r.best.localityFraction * 100).toFixed(0)}%`)
}

{
  const r = JSON.parse(
    run_factorization_search_json(
      JSON.stringify({
        kind: 'shuffled_chain',
        n: 6,
        field: 1.5,
        seed: 4242,
        topK: 3,
        inputMode: 'spectrum',
        eigenstateCount: 3,
      }),
    ),
  )
  console.log('\nfactorization spectrum:')
  ok(`scorer=${r.scorerUsed} recovered=${r.recoveredIdentity}`)
}

// --- Light cone compare ---
{
  const r = JSON.parse(
    run_light_cone_compare_json(
      JSON.stringify({ n: 10, field: 1.5, dt: 0.2, steps: 16, seed: 42 }),
    ),
  )
  console.log('\nlightConeCompare:')
  ok(`v_lattice=${r.comparison.lattice.velocity.toFixed(3)} ratio=${r.comparison.velocityRatio.toFixed(3)}`)
}

// --- Modular clock ---
{
  const r = JSON.parse(
    run_modular_dual_clock_json(
      JSON.stringify({ n: 12, field: 1.2, dt: 0.15, steps: 48, modularSlices: 12 }),
    ),
  )
  console.log('\nmodularDualClock:')
  ok(`syncR² mod=${r.syncR2Modular.toFixed(4)}`)
}

// --- Scattering (lite) ---
{
  const r = JSON.parse(
    run_scattering_json(
      JSON.stringify({
        n: 12,
        field: 0.7,
        dt: 0.12,
        steps: 40,
        defectSites: [3, 8],
        lite: true,
      }),
    ),
  )
  console.log('\nscattering (lite):')
  if (r.slices.length !== 0) fail('lite mode should omit slices')
  else ok(`crossed=${r.crossed} minSep=${r.minSeparation.toFixed(3)}`)
}

// --- Scattering (full) ---
{
  const r = JSON.parse(
    run_scattering_json(
      JSON.stringify({
        n: 12,
        field: 0.7,
        dt: 0.12,
        steps: 40,
        defectSites: [3, 8],
        lite: false,
      }),
    ),
  )
  console.log('\nscattering (full):')
  if (r.slices.length !== 40) fail(`expected 40 slices, got ${r.slices.length}`)
  else ok(`${r.slices.length} slices, worldlines=${r.worldlines.length}`)
}

console.log(failures === 0 ? '\nAll WASM checks passed.' : `\n${failures} check(s) failed.`)
process.exit(failures === 0 ? 0 : 1)
