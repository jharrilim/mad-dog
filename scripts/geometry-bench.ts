/**
 * Geometry dimension vs lattice sweep (Phase 2 S2).
 * Run: npm run bench:geometry
 */

import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import {
  initSync,
  run_geometry_dim_sweep_json,
  run_geometry_stability_json,
} from '../src/sim/wasm/pkg/mad_dog_sim.js'
import type { GeometryDimSweepResult } from '../src/sim/types.ts'

const __dirname = dirname(fileURLToPath(import.meta.url))
initSync({ module: readFileSync(join(__dirname, '../src/sim/wasm/pkg/mad_dog_sim_bg.wasm')) })

let failures = 0

console.warn('[mad-dog geometry] Phase-2 dim sweep + cube quench dim stability.')

const sweep = JSON.parse(run_geometry_dim_sweep_json('{}')) as GeometryDimSweepResult
console.log(`\nDim sweep: ${sweep.passed}/${sweep.total} passed`)
for (const c of sweep.cases) {
  const status = c.passed ? 'OK' : 'FAIL'
  if (!c.passed) failures++
  console.log(
    `  ${c.label}: dim=${c.emergentDim} expected≥${c.expectedDim} embed=${c.embeddingCorrelation.toFixed(3)} ${status}`,
  )
}
if (!sweep.allPassed) failures++

const cube = JSON.parse(
  run_geometry_stability_json(
    JSON.stringify({
      kind: 'cube',
      rows: 2,
      cols: 2,
      lz: 3,
      field: 1.5,
      dt: 0.2,
      steps: 20,
      xi: 1.0,
      seed: 42,
    }),
  ),
)
console.log('\nCube 2×2×3 quench stability:')
const cubeOk = cube.geometryStable && cube.dimStable
if (!cubeOk) failures++
console.log(
  `  rho=${cube.meanRankCorrelation.toFixed(3)} dimMean=${cube.meanEmergentDim.toFixed(2)} dimStable=${cube.dimStable} ${cubeOk ? 'OK' : 'FAIL'}`,
)

console.log(failures === 0 ? '\nAll geometry benchmarks passed.' : `\n${failures} case(s) failed.`)
process.exit(failures === 0 ? 0 : 1)
