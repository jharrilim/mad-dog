/**
 * Phase 8 — stabilizer code rate vs lab noise floors.
 */

import {
  run_stabilizer_search_json,
  run_eft_dimension_json,
} from '../src/sim/wasm/pkg/mad_dog_sim.js'
import type { EftDimensionResult, StabilizerSearchResult } from '../src/sim/types.ts'
import { ensureWasm, writeExport } from './observational-common.ts'
import { IR_NOISE_FLOORS } from './observational-constants.ts'

const COUPLINGS = [0.5, 0.9, 1.2] as const

export function runObservationalIr(): { exportPath: string } {
  ensureWasm()

  const sweep = COUPLINGS.map((coupling) => {
    const cfg = {
      n: 8,
      field: 1.2,
      dt: 0.2,
      steps: 22,
      coupleStep: 7,
      coupling,
      seed: 4242,
      windowRadius: 2,
    }
    const stab = JSON.parse(run_stabilizer_search_json(JSON.stringify(cfg))) as StabilizerSearchResult
    const eft = JSON.parse(run_eft_dimension_json(JSON.stringify(cfg))) as EftDimensionResult
    const redundancyGap = 1 - stab.stabilizer.codeRate
    return {
      coupling,
      codeRate: stab.stabilizer.codeRate,
      codeDistance: stab.stabilizer.codeDistance,
      generatorCount: stab.stabilizer.generatorCount,
      measuredDofPerSite: eft.measuredDofPerSite,
      predictedDofPerSite: eft.predictedDofPerSite,
      dofAgreement: eft.dofAgreement,
      redundancyGap,
    }
  })

  const headroom = IR_NOISE_FLOORS.map((floor) => {
    const maxGap = Math.max(...sweep.map((s) => s.redundancyGap))
    return {
      floorId: floor.id,
      floor: floor.value,
      maxRedundancyGap: maxGap,
      aboveFloor: maxGap > floor.value,
    }
  })

  const payload = {
    generatedAt: new Date().toISOString(),
    couplingSweep: sweep,
    noiseFloors: IR_NOISE_FLOORS,
    headroom,
    note: 'redundancyGap := 1 − codeRate; compare to per-cycle logical error order-of-magnitude.',
  }

  const exportPath = writeExport('ir-subspace', payload)

  console.log('\n=== IR subspace vs lab noise floors ===')
  for (const s of sweep) {
    console.log(
      `  coupling=${s.coupling}: rate=${s.codeRate.toFixed(3)} d=${s.codeDistance} DOF/site=${s.measuredDofPerSite.toFixed(2)} gap=${s.redundancyGap.toFixed(3)}`,
    )
  }
  console.log('\n  vs noise floors (redundancy gap > floor ⇒ toy code sparser than threshold):')
  for (const h of headroom) {
    console.log(`    ${h.floorId}: gap=${h.maxRedundancyGap.toFixed(4)} floor=${h.floor} above=${h.aboveFloor}`)
  }
  console.log(`\n  Wrote ${exportPath}`)

  return { exportPath }
}
