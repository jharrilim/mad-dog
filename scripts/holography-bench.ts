/**
 * Holography under dynamics bench (Phase 5 / S5).
 * Run: npm run bench:holography
 */

import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import {
  initSync,
  run_rt_quench_json,
  run_predictive_refinement_json,
  run_curvature_proxy_quench_json,
} from '../src/sim/wasm/pkg/mad_dog_sim.js'
import type {
  CurvatureProxyQuenchResult,
  PredictiveRefinementResult,
  RtQuenchResult,
} from '../src/sim/types.ts'

const __dirname = dirname(fileURLToPath(import.meta.url))
initSync({ module: readFileSync(join(__dirname, '../src/sim/wasm/pkg/mad_dog_sim_bg.wasm')) })

let failures = 0

console.warn('[mad-dog holography] Phase-5 RT quench, predictive refinement, curvature proxy.')

const quenchCfg = { n: 10, field: 1.5, dt: 0.2, steps: 16, seed: 7711 }

const rt = JSON.parse(run_rt_quench_json(JSON.stringify(quenchCfg))) as RtQuenchResult
console.log(`\nRT quench: ${rt.structuredDeviation ? 'OK' : 'FAIL'}`)
console.log(
  `  devDensityCorr=${rt.deviationDensityCorr.toFixed(3)} structured=${rt.structuredDeviation}`,
)
if (!rt.structuredDeviation) failures++

const predict = JSON.parse(
  run_predictive_refinement_json(
    JSON.stringify({ ...quenchCfg, steps: 18, deltaN: 2 }),
  ),
) as PredictiveRefinementResult
const predictOk = predict.leadTime > 0 && predict.lateSplitRecoverable
console.log(`\nPredictive refinement: ${predictOk ? 'OK' : 'FAIL'}`)
console.log(
  `  leadTime=${predict.leadTime} lateRecoverable=${predict.lateSplitRecoverable} warn=${predict.earlyWarningStep} fail=${predict.failureStep}`,
)
if (!predictOk) failures++

const proxy = JSON.parse(
  run_curvature_proxy_quench_json(JSON.stringify({ ...quenchCfg, xi: 1.0 })),
) as CurvatureProxyQuenchResult
console.log(`\nCurvature proxy: ${proxy.internallyConsistent ? 'OK' : 'FAIL'}`)
console.log(
  `  geoDensityCorr=${proxy.geoDensityCorr.toFixed(3)} crossCorr=${proxy.proxyCorrelation.toFixed(3)}`,
)
if (!proxy.internallyConsistent) failures++

if (failures > 0) {
  console.error(`\n${failures} check(s) failed.`)
  process.exit(1)
}
console.log('\nAll holography bench checks passed.')
