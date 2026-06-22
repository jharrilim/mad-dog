/**
 * Phase 8 — map lattice anisotropy ε(n) := speedCv to astrophysical LIV caps.
 * Run standalone or via observational-bridge.ts
 */

import {
  run_lorentz_scaling_json,
  run_dispersion_json,
  run_boost_invariance_json,
  run_light_cone_compare_json,
} from '../src/sim/wasm/pkg/mad_dog_sim.js'
import type { LorentzScalingResult } from '../src/sim/types.ts'
import { ensureWasm, writeExport } from './observational-common.ts'
import { LIV_BOUNDS } from './observational-constants.ts'

function computeGroupVelocityCov(modes: { groupVelocity: number }[]): number {
  if (modes.length < 2) return 0
  const mean = modes.reduce((s, m) => s + m.groupVelocity, 0) / modes.length
  if (mean < 1e-9) return 0
  const variance =
    modes.reduce((s, m) => s + (m.groupVelocity - mean) ** 2, 0) / modes.length
  return Math.sqrt(variance) / mean
}

export function runObservationalLorentz(): {
  epsilonByLattice: { label: string; rows: number; cols: number; epsilon: number }[]
  epsilonEffective: number
  headroom: { boundId: string; bound: number; epsilonEffective: number; ratio: number }[]
  dispersion: { linearR2: number; omegaSlope: number; linearAtSmallK: boolean; groupVelocityCov: number }
  boost: { relativeDelta: number; shapeInvariant: boolean }
  lightCone: { velocityRatio: number; ratioDeviation: number }
  exportPath: string
} {
  ensureWasm()

  const scaling = JSON.parse(run_lorentz_scaling_json('{}')) as LorentzScalingResult
  const epsilonByLattice = scaling.cases.map((c) => ({
    label: c.label,
    rows: c.rows,
    cols: c.cols,
    epsilon: c.speedCv,
  }))
  const speedCvMax = Math.max(...epsilonByLattice.map((e) => e.epsilon), 0)

  const dispersionRaw = JSON.parse(
    run_dispersion_json(JSON.stringify({ n: 16, field: 1.0, dt: 0.15, steps: 48, modes: 3 })),
  )
  const groupVelocityCov = computeGroupVelocityCov(dispersionRaw.modes ?? [])
  const boost = JSON.parse(
    run_boost_invariance_json(
      JSON.stringify({ rows: 4, cols: 4, field: 1.2, dt: 0.2, steps: 32, edgeSite: 0 }),
    ),
  )
  const lightCone = JSON.parse(
    run_light_cone_compare_json(
      JSON.stringify({ n: 10, field: 1.5, dt: 0.2, steps: 16, seed: 42 }),
    ),
  )
  const ratioDeviation = Math.abs(lightCone.comparison.velocityRatio - 1)

  // Cardinal speedCv can underflow to 0 when fronts are degenerate; use boost / dispersion / MI ratio too.
  const epsilonEffective = Math.max(
    speedCvMax,
    boost.relativeDelta,
    groupVelocityCov,
    ratioDeviation,
  )

  const headroom = LIV_BOUNDS.map((b) => ({
    boundId: b.id,
    bound: b.value,
    epsilonEffective,
    ratio: epsilonEffective / b.value,
  }))

  const payload = {
    generatedAt: new Date().toISOString(),
    proxy:
      'epsilonEffective := max(cardinal speedCv, boost relativeDelta, dispersion v_g CoV, |MI/lattice velocity ratio − 1|)',
    epsilonByLattice,
    epsilonEffective,
    covSmall: scaling.covSmall,
    covLarge: scaling.covLarge,
    covImproves: scaling.covImproves,
    headroom,
    bounds: LIV_BOUNDS,
    dispersion: {
      linearR2: dispersionRaw.linearR2,
      omegaSlope: dispersionRaw.omegaSlope,
      linearAtSmallK: dispersionRaw.linearAtSmallK,
      groupVelocityCov,
    },
    boost: {
      relativeDelta: boost.relativeDelta,
      shapeInvariant: boost.shapeInvariant,
    },
    lightCone: {
      velocityRatio: lightCone.comparison.velocityRatio,
      ratioDeviation,
    },
  }

  const exportPath = writeExport('lorentz', payload)

  console.log('\n=== Lorentz proxy ε(n) vs astrophysical caps ===')
  for (const e of epsilonByLattice) {
    console.log(`  ${e.label}: speedCv=${e.epsilon.toFixed(4)}`)
  }
  console.log(
    `  effective ε=${epsilonEffective.toFixed(4)} (boost=${boost.relativeDelta.toFixed(4)} vgCoV=${groupVelocityCov.toFixed(4)} MIratioΔ=${ratioDeviation.toFixed(4)})`,
  )
  console.log(
    `  scaling: covSmall=${scaling.covSmall.toFixed(4)} covLarge=${scaling.covLarge.toFixed(4)} improves=${scaling.covImproves}`,
  )
  console.log('\n  Headroom (ε_eff / bound) — values ≫ 1 mean toy lattice far above cap:')
  for (const h of headroom) {
    console.log(`    ${h.boundId}: ratio=${h.ratio.toExponential(2)}`)
  }
  console.log(`\n  Wrote ${exportPath}`)

  return {
    epsilonByLattice,
    epsilonEffective,
    headroom,
    dispersion: {
      linearR2: dispersionRaw.linearR2,
      omegaSlope: dispersionRaw.omegaSlope,
      linearAtSmallK: dispersionRaw.linearAtSmallK,
      groupVelocityCov,
    },
    boost: {
      relativeDelta: boost.relativeDelta,
      shapeInvariant: boost.shapeInvariant,
    },
    lightCone: {
      velocityRatio: lightCone.comparison.velocityRatio,
      ratioDeviation,
    },
    exportPath,
  }
}
