/**
 * WASM smoke + structural invariants (Rust is source of truth).
 * Run: npm run check:wasm
 */

import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import {
  initSync,
  wasm_sim_version,
  run_emergence_json,
  run_spacetime_json,
  run_holography_json,
  run_rt_mass_json,
  run_refinement_quench_json,
  run_refinement_n_compare_json,
  run_adaptive_refinement_json,
  run_predictive_refinement_json,
  run_rt_quench_json,
  run_curvature_proxy_quench_json,
  run_relational_time_json,
  run_universe_3d_json,
  run_universe_slice_json,
  run_light_cone_compare_json,
  run_modular_dual_clock_json,
  run_modular_multi_clock_json,
  run_multi_clock_json,
  run_simultaneity_json,
  run_lorentz_scaling_json,
  run_dispersion_json,
  run_boost_invariance_json,
  run_scattering_json,
  run_decoherence_quench_json,
  run_excitation_subspace_json,
  run_stabilizer_search_json,
  run_qecc_json,
  run_particle_stability_json,
  run_branch_born_json,
  run_eft_dimension_json,
  run_inplace_split_json,
  run_factorization_compare_json,
  run_holographic_bound_json,
  run_geometry_stability_json,
  run_geometry_dim_sweep_json,
  run_factorization_search_json,
} from '../src/sim/wasm/pkg/mad_dog_sim.js'

const __dirname = dirname(fileURLToPath(import.meta.url))
initSync({ module: readFileSync(join(__dirname, '../src/sim/wasm/pkg/mad_dog_sim_bg.wasm')) })

let failures = 0
function fail(msg: string) {
  console.log('  FAIL:', msg)
  failures++
}
function ok(msg: string) {
  console.log('  OK:', msg)
}

console.log('WASM version:', wasm_sim_version())

// --- Emergence ---
for (const config of [
  { kind: 'chain', n: 8, rows: 3, cols: 3, field: 1.5, seed: 12345 },
  { kind: 'grid', n: 9, rows: 3, cols: 3, field: 1.5, seed: 12345 },
  { kind: 'random', n: 8, rows: 3, cols: 3, field: 1.5, seed: 777 },
] as const) {
  const r = JSON.parse(run_emergence_json(JSON.stringify(config)))
  console.log(`\nemergence/${config.kind}:`)
  if (!Number.isFinite(r.energy)) fail('energy not finite')
  else ok(`energy=${r.energy.toFixed(4)}`)
  if (r.report.mi.length !== config.n) fail('MI size')
  else ok(`emergentDim=${r.report.mds.emergentDim}`)
}

// --- Spacetime ---
{
  const r = JSON.parse(
    run_spacetime_json(JSON.stringify({ n: 9, field: 0.8, dt: 0.2, steps: 20, seed: 7 })),
  )
  console.log('\nspacetime:')
  if (r.slices.length !== 20) fail(`expected 20 slices, got ${r.slices.length}`)
  else ok(`${r.slices.length} slices`)
  if (!r.lightCone?.velocity || r.lightCone.velocity <= 0) fail('lightCone velocity')
  else ok(`LR velocity=${r.lightCone.velocity.toFixed(3)}`)
  if (!r.worldline || r.worldline.length !== 20) fail('worldline length')
  else {
    const moved = r.worldline.some(
      (p: { site: number }, i: number) => i > 0 && p.site !== r.worldline[0].site,
    )
    if (!moved) fail('worldline should move in ordered phase')
    else ok('worldline tracks propagating defect')
  }
}

// --- Holography ---
{
  const r = JSON.parse(
    run_holography_json(JSON.stringify({ n: 10, field: 1.5, seed: 31337 })),
  )
  console.log('\nholography:')
  if (r.report.rtR2 < 0.85) fail(`RT R²=${r.report.rtR2}`)
  else ok(`RT R²=${r.report.rtR2.toFixed(4)}`)
}

// --- RT mass ---
{
  const r = JSON.parse(
    run_rt_mass_json(JSON.stringify({ n: 10, field: 1.5, seed: 7, strength: 1.0 })),
  )
  console.log('\nRT mass:')
  if (r.report.mass.rtSlope <= r.report.vacuum.rtSlope) fail('mass slope not above vacuum')
  else ok(`Δslope=${(r.report.mass.rtSlope - r.report.vacuum.rtSlope).toFixed(3)}`)

  const dense = JSON.parse(
    run_rt_mass_json(
      JSON.stringify({
        n: 10,
        field: 1.5,
        seed: 7,
        strength: 1.0,
        densitySweep: true,
        maxMassCount: 5,
      }),
    ),
  )
  const sweep = dense.report.densitySweep
  console.log('\nRT mass density sweep:')
  if (!sweep?.length) fail('densitySweep missing')
  else {
    const d0 = sweep[0].deltaSlope
    const dLast = sweep[sweep.length - 1].deltaSlope
    if (dLast <= d0 + 0.05) fail('Δslope did not grow with density')
    else ok(`ρ=0.1→${sweep[sweep.length - 1].density.toFixed(1)} Δslope=${d0.toFixed(2)}→${dLast.toFixed(2)}`)
  }
}

// --- Refinement ---
{
  const r = JSON.parse(
    run_refinement_quench_json(
      JSON.stringify({ n: 10, field: 1.5, dt: 0.2, steps: 16, seed: 7711 }),
    ),
  )
  console.log('\nrefinement quench:')
  const late = r.slices[r.slices.length - 1].diagnostics
  ok(`late pressure=${late.pressure.toFixed(3)}, decouplingLag=${r.decouplingLag}`)
}

{
  const r = JSON.parse(
    run_refinement_n_compare_json(
      JSON.stringify({ n: 10, deltaN: 2, field: 1.5, dt: 0.2, quenchStep: 12, seed: 7711 }),
    ),
  )
  console.log('\nrefinement n-compare:')
  ok(`small=${r.small.pressure.toFixed(3)} large=${r.large.pressure.toFixed(3)}`)
}

{
  const r = JSON.parse(
    run_adaptive_refinement_json(
      JSON.stringify({ n: 10, field: 1.5, dt: 0.2, steps: 18, seed: 7711, deltaN: 2 }),
    ),
  )
  console.log('\nadaptive refinement:')
  if (!r.splitEvent) fail('expected split trigger')
  else {
    ok(`trigger@${r.splitEvent.triggerStep} accepted=${r.splitEvent.accepted}`)
    if (!r.splitEvent.accepted) fail('split should relieve pressure')
    else ok(`Δpressure=${r.splitEvent.pressureDelta.toFixed(3)}`)
  }
}

{
  const r = JSON.parse(
    run_rt_quench_json(
      JSON.stringify({ n: 10, field: 1.5, dt: 0.2, steps: 16, seed: 7711 }),
    ),
  )
  console.log('\nRT quench:')
  if (!r.structuredDeviation) fail('expected structured RT deviation')
  else ok(`devDensityCorr=${r.deviationDensityCorr.toFixed(3)}`)
}

{
  const r = JSON.parse(
    run_predictive_refinement_json(
      JSON.stringify({ n: 10, field: 1.5, dt: 0.2, steps: 18, seed: 7711, deltaN: 2 }),
    ),
  )
  console.log('\npredictive refinement:')
  if (r.leadTime <= 0) fail('expected positive lead time')
  else ok(`leadTime=${r.leadTime}`)
  if (!r.lateSplitRecoverable) fail('late split should relieve pressure')
  else ok('late split recoverable')
}

{
  const r = JSON.parse(
    run_curvature_proxy_quench_json(
      JSON.stringify({ n: 10, field: 1.5, dt: 0.2, steps: 16, seed: 7711, xi: 1.0 }),
    ),
  )
  console.log('\ncurvature proxy:')
  if (!r.internallyConsistent) fail('proxy suite inconsistent')
  else ok(`geoDensityCorr=${r.geoDensityCorr.toFixed(3)}`)
}

// --- Relational time ---
{
  const r = JSON.parse(
    run_relational_time_json(
      JSON.stringify({ n: 9, field: 1, dt: 0.2, steps: 40, clockSite: 0, physicalSlices: 15 }),
    ),
  )
  console.log('\nrelational time:')
  ok(`syncR²=${r.syncR2.toFixed(4)}`)
}

// --- Multi-clock network ---
{
  const n = 9
  const r = JSON.parse(
    run_multi_clock_json(
      JSON.stringify({
        kind: 'chain',
        n,
        field: 1,
        dt: 0.2,
        steps: 40,
        clockSites: [0, Math.floor(n / 2), n - 1],
        physicalSlices: 15,
      }),
    ),
  )
  console.log('\nmultiClock (chain):')
  if (r.defectUniformR2 <= 0.95) fail(`defectUniformR2=${r.defectUniformR2}`)
  else ok(`defectUniformR2=${r.defectUniformR2.toFixed(3)}`)
  if (r.minPairwiseR2 >= 0.95) fail('network should not be globally consistent')
  else ok(`minPairwiseR2=${r.minPairwiseR2.toFixed(3)}, inconsistentPairs=${r.inconsistentPairs}`)
}

{
  const r = JSON.parse(
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
  console.log('\nmultiClock (grid 3x3):')
  if (r.defectUniformR2 <= 0.95) fail(`defectUniformR2=${r.defectUniformR2}`)
  else ok(`defectUniformR2=${r.defectUniformR2.toFixed(3)} minPair=${r.minPairwiseR2.toFixed(3)}`)
}

{
  const r = JSON.parse(
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
  console.log('\nsimultaneity surfaces:')
  if (!r.bendDetected) fail('foliation should bend between defect and edge clocks')
  else ok(`meanSkew=${r.meanTauSkew.toFixed(3)} slopeDelta=${r.slopeDelta.toFixed(3)}`)
}

// --- Universe ---
{
  const r = JSON.parse(
    run_universe_3d_json(JSON.stringify({ lx: 2, ly: 2, lz: 2, field: 1, dt: 0.25, steps: 12 })),
  )
  console.log('\nuniverse 3d:')
  const dim3 = r.spacetime.slices[0].coords.every((c: number[]) => c.length >= 3)
  if (!dim3) fail('coords not 3D')
  else ok(`LR velocity=${r.lightCone.velocity.toFixed(3)}`)
}

{
  const r = JSON.parse(
    run_universe_slice_json(
      JSON.stringify({ lx: 2, ly: 2, lz: 2, field: 1.5, dt: 0.25, steps: 12, k: 3 }),
    ),
  )
  console.log('\nuniverse slice (2x2x2 quench k=3):')
  if (r.report.mds.emergentDim < 3) fail(`expected dim≥3, got ${r.report.mds.emergentDim}`)
  else ok(`emergentDim=${r.report.mds.emergentDim}`)
}

{
  const r = JSON.parse(
    run_emergence_json(
      JSON.stringify({ kind: 'cube', lx: 2, ly: 2, lz: 3, field: 1.5, seed: 42 }),
    ),
  )
  console.log('\ncube ground (2x2x3):')
  if (r.report.mds.emergentDim < 3) fail(`expected dim≥3, got ${r.report.mds.emergentDim}`)
  else ok(`emergentDim=${r.report.mds.emergentDim}`)
}

{
  const r = JSON.parse(
    run_universe_slice_json(
      JSON.stringify({ lx: 2, ly: 2, lz: 3, field: 1.5, dt: 0.25, steps: 12, k: 3 }),
    ),
  )
  console.log('\nuniverse slice (2x2x3 quench k=3):')
  if (r.report.mds.emergentDim < 3) fail(`expected dim≥3, got ${r.report.mds.emergentDim}`)
  else ok(`emergentDim=${r.report.mds.emergentDim}`)
}

// --- Factorization ---
{
  const r = JSON.parse(
    run_factorization_search_json(
      JSON.stringify({ kind: 'shuffled_chain', n: 6, field: 1.5, seed: 4242, topK: 3 }),
    ),
  )
  console.log('\nfactorization (n=6):')
  if (!r.recoveredIdentity) fail('identity not recovered')
  else ok(`score=${r.best.score.toFixed(3)} locality=${(r.best.localityFraction * 100).toFixed(0)}%`)
}

{
  const cube = JSON.parse(
    run_factorization_search_json(
      JSON.stringify({
        kind: 'shuffled_cube',
        n: 8,
        field: 1.5,
        seed: 4242,
        topK: 3,
        graphKind: 'cube',
        lx: 2,
        ly: 2,
        lz: 2,
        searchMethod: 'exact',
      }),
    ),
  )
  const torus = JSON.parse(
    run_factorization_search_json(
      JSON.stringify({
        kind: 'shuffled_torus',
        n: 4,
        field: 1.5,
        seed: 4242,
        topK: 3,
        graphKind: 'torus',
        rows: 2,
        cols: 2,
        searchMethod: 'exact',
      }),
    ),
  )
  console.log('\nfactorization cube/torus:')
  if (!cube.recoveredIdentity || !torus.recoveredIdentity) {
    fail(`cube=${cube.recoveredIdentity} torus=${torus.recoveredIdentity}`)
  } else {
    ok(
      `cube loc=${(cube.best.localityFraction * 100).toFixed(0)}% torus loc=${(torus.best.localityFraction * 100).toFixed(0)}%`,
    )
  }
}

{
  const r = JSON.parse(
    run_factorization_search_json(
      JSON.stringify({
        kind: 'shuffled_chain',
        n: 6,
        field: 1.5,
        seed: 4242,
        topK: 3,
        inputMode: 'spectrum',
        eigenstateCount: 3,
      }),
    ),
  )
  console.log('\nfactorization spectrum:')
  if (!r.recoveredIdentity) fail('spectrum identity not recovered')
  else ok(`scorer=${r.scorerUsed} permMatch=${r.permMatchDistance ?? '?'} score=${r.best.score.toFixed(3)}`)
}

{
  const r = JSON.parse(
    run_factorization_search_json(
      JSON.stringify({
        kind: 'shuffled_xx_chain',
        n: 6,
        field: 1.5,
        seed: 4242,
        topK: 3,
        inputMode: 'spectrum',
        eigenstateCount: 3,
        searchMethod: 'exact',
      }),
    ),
  )
  console.log('\nfactorization XX spectrum:')
  if (!r.recoveredIdentity) fail('XX spectrum identity not recovered')
  else ok(`score=${r.best.score.toFixed(3)}`)
}

// --- Light cone compare ---
{
  const r = JSON.parse(
    run_light_cone_compare_json(
      JSON.stringify({ n: 10, field: 1.5, dt: 0.2, steps: 16, seed: 42 }),
    ),
  )
  console.log('\nlightConeCompare:')
  ok(`v_lattice=${r.comparison.lattice.velocity.toFixed(3)} ratio=${r.comparison.velocityRatio.toFixed(3)}`)
}

// --- Modular clock ---
{
  const r = JSON.parse(
    run_modular_dual_clock_json(
      JSON.stringify({ n: 12, field: 1.2, dt: 0.15, steps: 48, modularSlices: 12 }),
    ),
  )
  console.log('\nmodularDualClock:')
  ok(`syncR² mod=${r.syncR2Modular.toFixed(4)}`)
}

{
  const r = JSON.parse(
    run_modular_multi_clock_json(
      JSON.stringify({
        kind: 'grid',
        rows: 3,
        cols: 3,
        field: 1,
        dt: 0.2,
        steps: 40,
        modularSlices: 12,
      }),
    ),
  )
  console.log('\nmodularMultiClock (grid 3x3):')
  if (r.minPairwiseR2 >= 0.95) fail('network should not be globally consistent with uniform Δt')
  else ok(`minPairwiseR2=${r.minPairwiseR2.toFixed(3)} defectUniform=${r.defectUniformR2.toFixed(3)}`)
}

// --- Scattering (lite) ---
{
  const r = JSON.parse(
    run_scattering_json(
      JSON.stringify({
        n: 12,
        field: 0.7,
        dt: 0.12,
        steps: 40,
        defectSites: [3, 8],
        lite: true,
      }),
    ),
  )
  console.log('\nscattering (lite):')
  if (r.slices.length !== 0) fail('lite mode should omit slices')
  else ok(`crossed=${r.crossed} minSep=${r.minSeparation.toFixed(3)} bothMoved=${r.bothMoved}`)
  if (!r.phaseStable) fail(`phase should stabilize post-interaction (residual=${r.postInteractionPhaseStd})`)
  else ok(`phaseResidual=${r.postInteractionPhaseStd.toFixed(3)}`)
  if (!r.overlapDetected) fail('overlap should be detected on default scattering demo')
  else ok(`overlapStep=${r.overlapStep}`)
  if (!(r.interactionPhaseShift > 0.05 || r.interactionPhaseShift < -0.05))
    fail(`interaction phase shift too small (${r.interactionPhaseShift})`)
  else ok(`phaseShift=${r.interactionPhaseShift.toFixed(3)}`)
  if (!Number.isFinite(r.separationTimeDelay)) fail('separation time delay should be finite')
  else ok(`timeDelay=${r.separationTimeDelay.toFixed(3)}`)
  if (!(r.effectiveMass > 0.01)) fail(`effective mass too small (${r.effectiveMass})`)
  else ok(`m_eff=${r.effectiveMass.toFixed(3)} v_ratio=${r.velocityDispersionRatio.toFixed(2)}`)
  if (r.separationSeries?.length !== 40) fail('separation series length')
  else ok(`separationSeries=${r.separationSeries.length}`)
  if (r.kind !== 'chain') fail(`expected kind=chain, got ${r.kind}`)
}

// --- Scattering grid (lite) ---
{
  const r = JSON.parse(
    run_scattering_json(
      JSON.stringify({
        kind: 'grid',
        rows: 3,
        cols: 3,
        n: 9,
        field: 0.7,
        dt: 0.12,
        steps: 40,
        defectSites: [0, 8],
        lite: true,
      }),
    ),
  )
  console.log('\nscattering grid (lite):')
  if (r.kind !== 'grid') fail(`expected kind=grid, got ${r.kind}`)
  else ok(`label=${r.label}`)
  if (!r.bothMoved) fail('grid: both defects should move')
  else ok(`bothMoved minSep=${r.minSeparation.toFixed(2)}`)
  if (r.layoutPositions?.length !== 9) fail('grid layout positions')
  else ok(`layout=${r.layoutPositions.length} sites`)
}

// --- Scattering (full) ---
{
  const r = JSON.parse(
    run_scattering_json(
      JSON.stringify({
        n: 12,
        field: 0.7,
        dt: 0.12,
        steps: 40,
        defectSites: [3, 8],
        lite: false,
      }),
    ),
  )
  console.log('\nscattering (full):')
  if (r.slices.length !== 40) fail(`expected 40 slices, got ${r.slices.length}`)
  else ok(`${r.slices.length} slices, worldlines=${r.worldlines.length}`)
}

// --- Decoherence quench ---
{
  const r = JSON.parse(
    run_decoherence_quench_json(
      JSON.stringify({
        n: 8,
        field: 1.2,
        dt: 0.2,
        steps: 22,
        coupleStep: 7,
        coupling: 0.9,
        seed: 4242,
      }),
    ),
  )
  console.log('\ndecoherence quench:')
  if (!r.branchesDistinguishable) fail('branches not distinguishable')
  else
    ok(
      `sharpen=${r.sharpenRatio.toFixed(2)} p0=${r.branches[0].weight.toFixed(2)} branches=${r.branches.length}`,
    )
}

// --- Excitation subspace probe ---
{
  const r = JSON.parse(
    run_excitation_subspace_json(
      JSON.stringify({
        n: 8,
        field: 1.2,
        dt: 0.2,
        steps: 22,
        coupleStep: 7,
        coupling: 0.9,
        seed: 4242,
        windowRadius: 2,
      }),
    ),
  )
  console.log('\nexcitation subspace:')
  if (!r.codeLike) fail('branch excitation not code-like')
  else
    ok(
      `gain=${r.sharpnessGain.toFixed(2)} rankRed=${r.rankReduction.toFixed(2)} overlap=${r.branchOverlap.toFixed(3)}`,
    )
}

const excitationCfg = {
  n: 8,
  field: 1.2,
  dt: 0.2,
  steps: 22,
  coupleStep: 7,
  coupling: 0.9,
  seed: 4242,
  windowRadius: 2,
}

{
  const r = JSON.parse(run_stabilizer_search_json(JSON.stringify(excitationCfg)))
  console.log('\nstabilizer search:')
  if (!r.stabilizer.stabilizerFound) fail('no stabilizer generators')
  else ok(`generators=${r.stabilizer.generatorCount} distance=${r.stabilizer.codeDistance}`)
}

{
  const xx = JSON.parse(
    run_qecc_json(JSON.stringify({ ...excitationCfg, model: 'xx' })),
  )
  const heisenberg = JSON.parse(
    run_qecc_json(
      JSON.stringify({
        n: 6,
        field: 2.0,
        dt: 0.2,
        steps: 22,
        coupleStep: 5,
        coupling: 0.85,
        seed: 4242,
        windowRadius: 3,
        model: 'heisenberg',
      }),
    ),
  )
  console.log('\nQECC non-TFIM:')
  if (
    !xx.qecc.codeSubspaceFound ||
    xx.qecc.fidelitySelectivity < 1.1 ||
    !heisenberg.qecc.codeSubspaceFound ||
    heisenberg.qecc.fidelitySelectivity < 1.1
  ) {
    fail(
      `xx sel=${xx.qecc.fidelitySelectivity} heisenberg sel=${heisenberg.qecc.fidelitySelectivity}`,
    )
  } else {
    ok(
      `xx sel=${xx.qecc.fidelitySelectivity.toFixed(2)}× heisenberg sel=${heisenberg.qecc.fidelitySelectivity.toFixed(2)}×`,
    )
  }
}

{
  const r = JSON.parse(
    run_particle_stability_json(JSON.stringify({ n: 10, dt: 0.2, steps: 24 })),
  )
  console.log('\nparticle stability:')
  if (!r.orderedLongerLived) fail('ordered phase not more localized')
  else ok(`ordered=${r.ordered.localizationFraction.toFixed(2)}`)
}

{
  const r = JSON.parse(
    run_branch_born_json(
      JSON.stringify({ n: 8, field: 1.2, dt: 0.2, steps: 22, coupleStep: 7 }),
    ),
  )
  console.log('\nbranch Born:')
  if (!r.bornConsistent) fail('Born weights inconsistent')
  else ok(`entropyCorr=${r.entropyOverlapCorr.toFixed(2)}`)
}

{
  const r = JSON.parse(run_eft_dimension_json(JSON.stringify(excitationCfg)))
  console.log('\nEFT dimension:')
  if (!r.dofAgreement) fail('EFT DOF mismatch')
  else ok(`measured=${r.measuredDofPerSite.toFixed(2)} predicted=${r.predictedDofPerSite.toFixed(2)}`)
}

{
  const r = JSON.parse(
    run_inplace_split_json(
      JSON.stringify({ n: 10, field: 1.5, dt: 0.2, steps: 18, seed: 7711, deltaN: 2 }),
    ),
  )
  console.log('\nin-place split:')
  if (!r.splitEvent?.inPlaceImproves) fail('in-place split did not relieve pressure')
  else ok(`step=${r.splitEvent.triggerStep} deltaP=${r.splitEvent.pressureDelta.toFixed(3)}`)
}

{
  const r = JSON.parse(
    run_factorization_compare_json(
      JSON.stringify({ nSmall: 8, field: 1.5, dt: 0.2, step: 12, seed: 4242, deltaN: 2 }),
    ),
  )
  console.log('\nfactorization compare (same |ψ⟩, different n):')
  if (!r.embeddingFaithful || r.roundtripFidelity < 0.99) {
    fail(`embed/truncate not faithful (F=${r.roundtripFidelity}, faithful=${r.embeddingFaithful})`)
  } else {
    ok(`F=${r.roundtripFidelity.toFixed(4)} Δloc_rt=${r.localityDriftRoundtrip.toFixed(3)} Δloc_emb=${r.localityDriftEmbed.toFixed(3)}`)
  }
}

{
  const r = JSON.parse(run_holographic_bound_json(JSON.stringify({ field: 1.5, nMin: 6, nMax: 12 })))
  console.log('\nholographic bound:')
  if (r.nMin == null || !r.boundScales) fail('no finite holographic n_min')
  else ok(`nMin=${r.nMin} saturation=${r.nSaturation}`)
}

// --- Gauge-free geometry stability ---
{
  const r = JSON.parse(
    run_geometry_stability_json(
      JSON.stringify({
        kind: 'chain',
        n: 10,
        field: 1.2,
        dt: 0.2,
        steps: 20,
        xi: 1.0,
        seed: 42,
      }),
    ),
  )
  console.log('\ngeometry stability (chain):')
  if (!r.geometryStable) fail('ordered-phase MI ranking unstable')
  else
    ok(
      `rho=${r.meanRankCorrelation.toFixed(3)} embed=${r.meanEmbeddingCorrelation.toFixed(3)} dimStd=${r.dimStd.toFixed(2)}`,
    )
}

{
  const r = JSON.parse(
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
  console.log('\ngeometry stability (cube 2x2x3):')
  if (!r.geometryStable || !r.dimStable) fail('cube geometry unstable')
  else
    ok(
      `rho=${r.meanRankCorrelation.toFixed(3)} dimMean=${r.meanEmergentDim.toFixed(1)} expected=${r.expectedDim}`,
    )
}

{
  const r = JSON.parse(run_geometry_dim_sweep_json('{}'))
  console.log('\ngeometry dim sweep:')
  if (!r.allPassed) fail(`dim sweep ${r.passed}/${r.total}`)
  else ok(`${r.passed}/${r.total} lattice cases passed`)
}

{
  const r = JSON.parse(run_lorentz_scaling_json('{}'))
  console.log('\nlorentz scaling:')
  if (!r.allPassed) fail(`scaling covSmall=${r.covSmall} covLarge=${r.covLarge}`)
  else ok(`covSmall=${r.covSmall.toFixed(3)} covLarge=${r.covLarge.toFixed(3)} improves=${r.covImproves}`)
}

{
  const r = JSON.parse(
    run_dispersion_json(JSON.stringify({ n: 16, field: 1.0, dt: 0.15, steps: 48, modes: 3 })),
  )
  console.log('\ndispersion:')
  if (!r.linearAtSmallK) fail('omega(k) should be linear at small k')
  else ok(`slope=${r.omegaSlope.toFixed(3)} R²=${r.linearR2.toFixed(3)} m_eff=${r.effectiveMass.toFixed(3)}`)
}

console.log(failures === 0 ? '\nAll WASM checks passed.' : `\n${failures} check(s) failed.`)
process.exit(failures === 0 ? 0 : 1)
