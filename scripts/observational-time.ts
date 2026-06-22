/**
 * Phase 8 — multi-clock desync export (problem-of-time metaphor).
 */

import {
  run_multi_clock_json,
  run_modular_dual_clock_json,
  run_simultaneity_json,
} from '../src/sim/wasm/pkg/mad_dog_sim.js'
import type { MultiClockResult, SimultaneityResult } from '../src/sim/types.ts'
import { ensureWasm, writeExport } from './observational-common.ts'

export function runObservationalTime(): { exportPath: string } {
  ensureWasm()

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
  ) as MultiClockResult

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
  ) as MultiClockResult

  const modular = JSON.parse(
    run_modular_dual_clock_json(
      JSON.stringify({ n: 12, field: 1.2, dt: 0.15, steps: 48, modularSlices: 12 }),
    ),
  )

  const simultaneity = JSON.parse(
    run_simultaneity_json(
      JSON.stringify({
        n: 9,
        field: 1,
        dt: 0.2,
        steps: 40,
        clockA: 4,
        clockB: 8,
      }),
    ),
  ) as SimultaneityResult

  const payload = {
    generatedAt: new Date().toISOString(),
    metaphor:
      'No global τ: each clock subsystem induces a foliation; desync quantifies frozen-formalism multiplicity.',
    grid: {
      minPairwiseR2: grid.minPairwiseR2,
      defectUniformR2: grid.defectUniformR2,
      inconsistentPairs: grid.inconsistentPairs,
      clockCount: grid.clocks.length,
    },
    cube: {
      minPairwiseR2: cube.minPairwiseR2,
      defectUniformR2: cube.defectUniformR2,
      inconsistentPairs: cube.inconsistentPairs,
      clockCount: cube.clocks.length,
    },
    modular: {
      syncR2Modular: modular.syncR2Modular,
      syncR2Z: modular.syncR2Z,
      minModularUniformR2: modular.minModularUniformR2,
      minZEdgeUniformR2: modular.minZEdgeUniformR2,
    },
    simultaneity: {
      bendDetected: simultaneity.bendDetected,
      meanTauSkew: simultaneity.meanTauSkew,
      maxTauSkew: simultaneity.maxTauSkew,
      slopeDelta: simultaneity.slopeDelta,
      slices: simultaneity.slices.map((s) => ({
        k: s.k,
        tauSkew: s.tauSkew,
        emergentX: s.emergentX,
        emergentY: s.emergentY,
      })),
    },
  }

  const exportPath = writeExport('time', payload)

  console.log('\n=== Problem of time (multi-clock desync) ===')
  console.log(
    `  grid 3×3: minPairR²=${grid.minPairwiseR2.toFixed(3)} defectUniform=${grid.defectUniformR2.toFixed(3)}`,
  )
  console.log(
    `  cube 2×2×3: minPairR²=${cube.minPairwiseR2.toFixed(3)} inconsistentPairs=${cube.inconsistentPairs}`,
  )
  console.log(
    `  modular: minModUniform=${modular.minModularUniformR2.toFixed(3)} minZEdge=${modular.minZEdgeUniformR2.toFixed(3)}`,
  )
  console.log(
    `  simultaneity: bend=${simultaneity.bendDetected} meanSkew=${simultaneity.meanTauSkew.toFixed(3)} slopeΔ=${simultaneity.slopeDelta.toFixed(3)}`,
  )
  console.log(`\n  Wrote ${exportPath}`)

  return { exportPath }
}
