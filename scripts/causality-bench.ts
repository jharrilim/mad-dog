/**
 * Causality / Lorentz bench (Phase 4 S4→S9).
 * Run: npm run bench:causality
 */

import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import {
  initSync,
  run_lorentz_scaling_json,
  run_dispersion_json,
  run_boost_invariance_json,
  run_scattering_json,
} from '../src/sim/wasm/pkg/mad_dog_sim.js'
import type { LorentzScalingResult } from '../src/sim/types.ts'

const __dirname = dirname(fileURLToPath(import.meta.url))
initSync({ module: readFileSync(join(__dirname, '../src/sim/wasm/pkg/mad_dog_sim_bg.wasm')) })

let failures = 0

console.warn('[mad-dog causality] Phase-4 Lorentz scaling, dispersion, boost, scattering phase.')

const scaling = JSON.parse(run_lorentz_scaling_json('{}')) as LorentzScalingResult
console.log(`\nLorentz scaling: ${scaling.allPassed ? 'OK' : 'FAIL'}`)
for (const c of scaling.cases) {
  console.log(`  ${c.label}: CoV=${c.speedCv.toFixed(3)} ${c.passed ? 'OK' : 'FAIL'}`)
  if (!c.passed) failures++
}
if (!scaling.covImproves) failures++
console.log(`  covSmall=${scaling.covSmall.toFixed(3)} covLarge=${scaling.covLarge.toFixed(3)} improves=${scaling.covImproves}`)

const dispersion = JSON.parse(
  run_dispersion_json(JSON.stringify({ n: 16, field: 1.0, dt: 0.15, steps: 48, modes: 3 })),
)
console.log('\nDispersion:')
const dispOk = dispersion.linearAtSmallK
if (!dispOk) failures++
console.log(
  `  slope=${dispersion.omegaSlope.toFixed(3)} R²=${dispersion.linearR2.toFixed(3)} linear=${dispOk ? 'OK' : 'FAIL'}`,
)

const boost = JSON.parse(
  run_boost_invariance_json(JSON.stringify({ rows: 4, cols: 4, field: 1.2, dt: 0.2, steps: 32, edgeSite: 0 })),
)
console.log('\nBoost invariance:')
if (!boost.shapeInvariant) failures++
console.log(
  `  relDelta=${boost.relativeDelta.toFixed(3)} ${boost.shapeInvariant ? 'OK' : 'FAIL'}`,
)

const scatter = JSON.parse(
  run_scattering_json(
    JSON.stringify({
      n: 12,
      field: 0.7,
      dt: 0.12,
      steps: 40,
      defectSites: [3, 8],
      lite: true,
      taylorOrder: 4,
    }),
  ),
)
console.log('\nScattering phase:')
if (!scatter.phaseStable) failures++
console.log(
  `  residualStd=${scatter.postInteractionPhaseStd.toFixed(3)} ${scatter.phaseStable ? 'OK' : 'FAIL'}`,
)
if (!scatter.overlapDetected) failures++
console.log(
  `  overlapStep=${scatter.overlapStep} phaseShift=${scatter.interactionPhaseShift.toFixed(3)} ${scatter.overlapDetected ? 'OK' : 'FAIL'}`,
)
if (!(scatter.interactionPhaseShift > 0.05 || scatter.interactionPhaseShift < -0.05)) failures++
if (!Number.isFinite(scatter.separationTimeDelay)) failures++
console.log(
  `  timeDelay=${scatter.separationTimeDelay.toFixed(3)} ${Number.isFinite(scatter.separationTimeDelay) ? 'OK' : 'FAIL'}`,
)

if (failures > 0) {
  console.error(`\n${failures} check(s) failed.`)
  process.exit(1)
}
console.log('\nAll causality bench checks passed.')
