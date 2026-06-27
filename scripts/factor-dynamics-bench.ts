/**
 * Factor count dynamics bench (Phase 7 / S10).
 * Run: npm run bench:factor
 */

import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import {
  initSync,
  run_inplace_split_json,
  run_holographic_bound_json,
  run_factor_count_dynamics_json,
  run_field_sweep_json,
} from '../src/sim/wasm/pkg/mad_dog_sim.js'
import type {
  HolographicBoundResult,
  InplaceSplitResult,
  FactorCountDynamicsResult,
  FieldSweepResult,
} from '../src/sim/types.ts'

const __dirname = dirname(fileURLToPath(import.meta.url))
initSync({ module: readFileSync(join(__dirname, '../src/sim/wasm/pkg/mad_dog_sim_bg.wasm')) })

let failures = 0

console.warn('[mad-dog factor] Phase-7 V′/W′ + AB factor-count dynamics + AC field-sweep holographic emergence.')

const split = JSON.parse(
  run_inplace_split_json(
    JSON.stringify({ n: 10, field: 1.5, dt: 0.2, steps: 18, seed: 7711, deltaN: 2 }),
  ),
) as InplaceSplitResult
const ev = split.splitEvent
const splitOk = ev?.inPlaceImproves ?? false
console.log(`\nIn-place split: ${splitOk ? 'OK' : 'FAIL'}`)
if (ev) {
  console.log(
    `  step=${ev.triggerStep} preP=${ev.pre.pressure.toFixed(3)} inPlaceP=${ev.inPlace.pressure.toFixed(3)}`,
  )
} else {
  console.log('  no split trigger')
}
if (!splitOk) failures++

const bound = JSON.parse(
  run_holographic_bound_json(JSON.stringify({ field: 1.5, nMin: 6, nMax: 12 })),
) as HolographicBoundResult
const boundOk = bound.nMin != null && bound.boundScales
console.log(`\nHolographic bound: ${boundOk ? 'OK' : 'FAIL'}`)
console.log(`  nMin=${bound.nMin} saturation=${bound.nSaturation} scales=${bound.boundScales}`)
if (!boundOk) failures++

// Falsification AB: n_opt_pressure(t) increases with the entanglement light cone.
const fcd = JSON.parse(
  run_factor_count_dynamics_json(
    JSON.stringify({ nStart: 4, nMax: 14, deltaN: 2, field: 1.5, dt: 0.1, steps: 40, seed: 7711 }),
  ),
) as FactorCountDynamicsResult
const abOk = fcd.nOptPressureIncreases
console.log(`\nFalsification AB — factor count dynamics: ${abOk ? 'OK' : 'FAIL'}`)
console.log(
  `  n_opt_pressure_peak=${fcd.nOptPressurePeak} n_start=${fcd.nStart} n_max=${fcd.nMax}`,
)
// Print the staircase profile
const staircase = fcd.series
  .filter((_, i) => i % 4 === 0)
  .map((p) => `t=${p.t.toFixed(1)}→n=${p.nOptPressure}`)
  .join('  ')
console.log(`  staircase: ${staircase}`)
if (!abOk) failures++

// Falsification AC: holographic structure is absent in the ordered phase (h<1) and
// emerges near the TFIM critical point (h≈1.0). Skip locality for speed.
const sweep = JSON.parse(
  run_field_sweep_json(
    JSON.stringify({
      fields: [0.3, 0.5, 0.7, 1.0, 1.2, 1.5],
      nMin: 6,
      nMax: 12,
      skipLocality: false,
    }),
  ),
) as FieldSweepResult
const acOk = sweep.emergenceNearCritical && sweep.orderedPhaseNonholographic
console.log(`\nFalsification AC — field-sweep holographic emergence: ${acOk ? 'OK' : 'FAIL'}`)
console.log(
  `  h_emergence=${sweep.hHolographicEmergence} near_critical=${sweep.emergenceNearCritical} ordered_nonholo=${sweep.orderedPhaseNonholographic}`,
)
const fieldProfile = sweep.points
  .map((p) => `h=${p.field.toFixed(1)}→${p.scan.nMin ?? 'None'}`)
  .join('  ')
console.log(`  profile: ${fieldProfile}`)
if (!acOk) failures++

if (failures > 0) {
  console.error(`\n${failures} failure(s)`)
  process.exit(1)
}

console.log('\nAll factor-dynamics checks passed.')
