/**
 * Phase 8 — RT slope / ratio deviations vs AdS/CFT analogue references.
 */

import {
  run_holography_json,
  run_rt_mass_json,
  run_rt_quench_json,
} from '../src/sim/wasm/pkg/mad_dog_sim.js'
import type { RtQuenchResult } from '../src/sim/types.ts'
import { ensureWasm, writeExport } from './observational-common.ts'
import { HOLOGRAPHY_REFERENCES } from './observational-constants.ts'

export function runObservationalHolography(): { exportPath: string } {
  ensureWasm()

  const ground = JSON.parse(
    run_holography_json(JSON.stringify({ n: 10, field: 1.5, seed: 7 })),
  )
  const mass = JSON.parse(
    run_rt_mass_json(
      JSON.stringify({
        n: 10,
        field: 1.5,
        strength: 1.0,
        seed: 7,
        densitySweep: true,
        maxMassCount: 5,
      }),
    ),
  )
  const quench = JSON.parse(
    run_rt_quench_json(
      JSON.stringify({ n: 10, field: 1.5, dt: 0.2, steps: 16, seed: 7711 }),
    ),
  ) as RtQuenchResult

  const vacuumSlope = mass.report.vacuum.rtSlope as number
  const densitySweep = (mass.report.densitySweep ?? []) as {
    count: number
    density: number
    slope: number
    r2: number
    deltaSlope: number
  }[]

  const quenchSummary = {
    structuredDeviation: quench.structuredDeviation,
    deviationDensityCorr: quench.deviationDensityCorr,
    finalMeanRtRatioDev:
      quench.slices[quench.slices.length - 1]?.meanRtRatioDev ?? null,
    finalSlopeDeviation: quench.slices[quench.slices.length - 1]?.slopeDeviation ?? null,
  }

  const payload = {
    generatedAt: new Date().toISOString(),
    references: HOLOGRAPHY_REFERENCES,
    ground: {
      rtSlope: ground.report.rtSlope,
      rtR2: ground.report.rtR2,
      deltaFromAdsCft: ground.report.rtSlope - HOLOGRAPHY_REFERENCES.adsCftRtSlope,
    },
    vacuum: { rtSlope: vacuumSlope, deltaFromAdsCft: vacuumSlope - HOLOGRAPHY_REFERENCES.adsCftRtSlope },
    densitySweep: densitySweep.map((p) => ({
      density: p.density,
      rtSlope: p.slope,
      rtR2: p.r2,
      deltaSlope: p.deltaSlope,
      deltaFromAdsCft: p.slope - HOLOGRAPHY_REFERENCES.adsCftRtSlope,
    })),
    quench: quenchSummary,
  }

  const exportPath = writeExport('holography', payload)

  console.log('\n=== Holographic RT calibration ===')
  console.log(
    `  ground: slope=${ground.report.rtSlope.toFixed(3)} R²=${ground.report.rtR2.toFixed(3)} (ref=${HOLOGRAPHY_REFERENCES.adsCftRtSlope})`,
  )
  console.log(`  vacuum slope=${vacuumSlope.toFixed(3)}`)
  if (densitySweep.length) {
    const last = densitySweep[densitySweep.length - 1]
    console.log(
      `  max density: slope=${last.slope.toFixed(3)} Δslope=${last.deltaSlope.toFixed(3)} R²=${last.r2.toFixed(3)}`,
    )
  }
  console.log(
    `  quench: structured=${quench.structuredDeviation} ρ(dev,density)=${quench.deviationDensityCorr.toFixed(3)}`,
  )
  console.log(`\n  Wrote ${exportPath}`)

  return { exportPath }
}
