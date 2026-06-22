/**
 * Relational time bench (Phase 3 S3).
 * Run: npm run bench:time
 */

import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import {
  initSync,
  run_multi_clock_json,
  run_modular_dual_clock_json,
  run_simultaneity_json,
} from '../src/sim/wasm/pkg/mad_dog_sim.js'

const __dirname = dirname(fileURLToPath(import.meta.url))
initSync({ module: readFileSync(join(__dirname, '../src/sim/wasm/pkg/mad_dog_sim_bg.wasm')) })

let failures = 0

console.warn('[mad-dog time] Phase-3 relational time: grid/cube clocks, modular vs uniform, simultaneity bend.')

const grid = JSON.parse(
  run_multi_clock_json(
    JSON.stringify({
      kind: 'grid',
      rows: 3,
      cols: 3,
      field: 1,
      dt: 0.2,
      steps: 40,
      physicalSlices: 15,
    }),
  ),
)
const gridOk = grid.defectUniformR2 > 0.95 && grid.minPairwiseR2 < 0.95
console.log(
  `\nGrid 3×3 multi-clock: defectUniform=${grid.defectUniformR2.toFixed(3)} minPair=${grid.minPairwiseR2.toFixed(3)} ${gridOk ? 'OK' : 'FAIL'}`,
)
if (!gridOk) failures++

const cube = JSON.parse(
  run_multi_clock_json(
    JSON.stringify({
      kind: 'cube',
      rows: 2,
      cols: 2,
      lz: 3,
      field: 1,
      dt: 0.2,
      steps: 40,
      physicalSlices: 15,
    }),
  ),
)
const cubeOk = cube.defectUniformR2 > 0.95 && cube.minPairwiseR2 < 0.95
console.log(
  `Cube 2×2×3 multi-clock: defectUniform=${cube.defectUniformR2.toFixed(3)} minPair=${cube.minPairwiseR2.toFixed(3)} ${cubeOk ? 'OK' : 'FAIL'}`,
)
if (!cubeOk) failures++

const modular = JSON.parse(
  run_modular_dual_clock_json(
    JSON.stringify({ n: 12, field: 1.2, dt: 0.15, steps: 48, modularSlices: 12 }),
  ),
)
const modOk = modular.minModularUniformR2 < 0.95 && modular.minZEdgeUniformR2 < 0.95
console.log(
  `Modular vs uniform: minMod=${modular.minModularUniformR2.toFixed(3)} minZEdge=${modular.minZEdgeUniformR2.toFixed(3)} ${modOk ? 'OK' : 'FAIL'}`,
)
if (!modOk) failures++

const sim = JSON.parse(
  run_simultaneity_json(
    JSON.stringify({
      n: 9,
      field: 1,
      dt: 0.2,
      steps: 40,
      clockA: 4,
      clockB: 0,
      physicalSlices: 15,
      embedDim: 2,
      referenceSite: 0,
    }),
  ),
)
console.log(
  `Simultaneity bend: meanSkew=${sim.meanTauSkew.toFixed(3)} slopeDelta=${sim.slopeDelta.toFixed(3)} ${sim.bendDetected ? 'OK' : 'FAIL'}`,
)
if (!sim.bendDetected) failures++

if (failures > 0) {
  console.error(`\n${failures} check(s) failed.`)
  process.exit(1)
}
console.log('\nAll time bench checks passed.')
