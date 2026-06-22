/**
 * Matter / classicality bench (Phase 6 / S6–S8).
 * Run: npm run bench:matter
 */

import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import {
  initSync,
  run_stabilizer_search_json,
  run_particle_stability_json,
  run_branch_born_json,
  run_eft_dimension_json,
} from '../src/sim/wasm/pkg/mad_dog_sim.js'
import type {
  BranchBornResult,
  EftDimensionResult,
  ParticleStabilityResult,
  StabilizerSearchResult,
} from '../src/sim/types.ts'

const __dirname = dirname(fileURLToPath(import.meta.url))
initSync({ module: readFileSync(join(__dirname, '../src/sim/wasm/pkg/mad_dog_sim_bg.wasm')) })

let failures = 0

console.warn('[mad-dog matter] Phase-6 stabilizer, particle stability, Born weights, EFT DOF.')

const excitation = {
  n: 8,
  field: 1.2,
  dt: 0.2,
  steps: 22,
  coupleStep: 7,
  coupling: 0.9,
  seed: 4242,
  windowRadius: 2,
}

const stab = JSON.parse(
  run_stabilizer_search_json(JSON.stringify(excitation)),
) as StabilizerSearchResult
const stabOk = stab.stabilizer.stabilizerFound && stab.distanceScales
console.log(`\nStabilizer search: ${stabOk ? 'OK' : 'FAIL'}`)
console.log(
  `  generators=${stab.stabilizer.generatorCount} distance=${stab.stabilizer.codeDistance} scales=${stab.distanceScales}`,
)
if (!stabOk) failures++

const particle = JSON.parse(
  run_particle_stability_json(JSON.stringify({ n: 10, dt: 0.2, steps: 24 })),
) as ParticleStabilityResult
console.log(`\nParticle stability: ${particle.orderedLongerLived ? 'OK' : 'FAIL'}`)
console.log(
  `  ordered=${particle.ordered.localizationFraction.toFixed(3)} disordered=${particle.disordered.localizationFraction.toFixed(3)}`,
)
if (!particle.orderedLongerLived) failures++

const born = JSON.parse(
  run_branch_born_json(
    JSON.stringify({ n: 8, field: 1.2, dt: 0.2, steps: 22, coupleStep: 7 }),
  ),
) as BranchBornResult
console.log(`\nBranch Born: ${born.bornConsistent ? 'OK' : 'FAIL'}`)
console.log(
  `  entropyCorr=${born.entropyOverlapCorr.toFixed(3)} imbalanceCorr=${born.imbalanceOverlapCorr.toFixed(3)}`,
)
if (!born.bornConsistent) failures++

const eft = JSON.parse(
  run_eft_dimension_json(JSON.stringify(excitation)),
) as EftDimensionResult
console.log(`\nEFT dimension: ${eft.dofAgreement ? 'OK' : 'FAIL'}`)
console.log(
  `  measured=${eft.measuredDofPerSite.toFixed(3)} predicted=${eft.predictedDofPerSite.toFixed(3)}`,
)
if (!eft.dofAgreement) failures++

if (failures > 0) {
  console.error(`\n${failures} check(s) failed.`)
  process.exit(1)
}
console.log('\nAll matter bench checks passed.')
