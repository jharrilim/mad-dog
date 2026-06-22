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
} from '../src/sim/wasm/pkg/mad_dog_sim.js'
import type { HolographicBoundResult, InplaceSplitResult } from '../src/sim/types.ts'

const __dirname = dirname(fileURLToPath(import.meta.url))
initSync({ module: readFileSync(join(__dirname, '../src/sim/wasm/pkg/mad_dog_sim_bg.wasm')) })

let failures = 0

console.warn('[mad-dog factor] Phase-7 in-place split + holographic n_min.')

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

if (failures > 0) {
  console.error(`\n${failures} failure(s)`)
  process.exit(1)
}

console.log('\nAll factor-dynamics checks passed.')
