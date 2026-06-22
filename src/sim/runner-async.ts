/**
 * Async sim drivers — WASM worker only (no TypeScript calculation fallback).
 */

import {
  warnIfExpensiveFactorizationRefinement,
  warnIfExpensiveFactorizationSearch,
} from './factorization-warnings.ts'
import type {
  DualClockResult,
  FactorizationRefinementConfig,
  FactorizationRefinementResult,
  FactorizationSearchConfig,
  FactorizationSearchResult,
  FalsificationBatteryResult,
  HolographyRunConfig,
  HolographyRunResult,
  LightConeCompareResult,
  ModularDualClockConfig,
  ModularDualClockResult,
  MultiClockConfig,
  MultiClockResult,
  RefinementNCompareConfig,
  RefinementNCompareResult,
  RefinementQuenchConfig,
  RefinementQuenchResult,
  AdaptiveRefinementConfig,
  AdaptiveRefinementResult,
  CurvatureProxyQuenchConfig,
  CurvatureProxyQuenchResult,
  PredictiveRefinementResult,
  RelationalTimeConfig,
  RtQuenchConfig,
  RtQuenchResult,
  RtMassRunConfig,
  RtMassRunResult,
  RunConfig,
  RunResult,
  SimultaneityConfig,
  SimultaneityResult,
  LorentzScalingResult,
  DispersionConfig,
  DispersionResult,
  BoostInvarianceConfig,
  BoostInvarianceResult,
  ScatteringConfig,
  ScatteringResult,
  DecoherenceQuenchConfig,
  DecoherenceQuenchResult,
  ExcitationSubspaceConfig,
  ExcitationSubspaceResult,
  StabilizerSearchConfig,
  StabilizerSearchResult,
  ParticleStabilityConfig,
  ParticleStabilityResult,
  BranchBornConfig,
  BranchBornResult,
  EftDimensionConfig,
  EftDimensionResult,
  GeometryStabilityConfig,
  GeometryStabilityResult,
  SimBackend,
  Spacetime2DConfig,
  SpacetimeResult,
  SpacetimeRunConfig,
  Universe3DConfig,
  Universe3DResult,
  UniverseSliceConfig,
  UniverseSliceResult,
} from './types.ts'
import { callWasm, withBackend } from './wasm/client.ts'

async function runWasm<T extends { elapsedMs: number }>(
  method: Parameters<typeof callWasm>[0],
  config: unknown,
): Promise<T & { backend: SimBackend }> {
  const result = await callWasm<T>(method, config)
  return withBackend(result, 'wasm')
}

export type RunResultWithBackend = RunResult & { backend: SimBackend }
export type SpacetimeResultWithBackend = SpacetimeResult & { backend: SimBackend }
export type LightConeCompareResultWithBackend = LightConeCompareResult & {
  backend: SimBackend
}
export type HolographyRunResultWithBackend = HolographyRunResult & {
  backend: SimBackend
}
export type RtMassRunResultWithBackend = RtMassRunResult & { backend: SimBackend }
export type RefinementQuenchResultWithBackend = RefinementQuenchResult & {
  backend: SimBackend
}
export type RefinementNCompareResultWithBackend = RefinementNCompareResult & {
  backend: SimBackend
}
export type AdaptiveRefinementResultWithBackend = AdaptiveRefinementResult & {
  backend: SimBackend
}
export type DualClockResultWithBackend = DualClockResult & { backend: SimBackend }
export type MultiClockResultWithBackend = MultiClockResult & { backend: SimBackend }
export type ModularDualClockResultWithBackend = ModularDualClockResult & {
  backend: SimBackend
}
export type ScatteringResultWithBackend = ScatteringResult & { backend: SimBackend }
export type DecoherenceQuenchResultWithBackend = DecoherenceQuenchResult & {
  backend: SimBackend
}
export type ExcitationSubspaceResultWithBackend = ExcitationSubspaceResult & {
  backend: SimBackend
}
export type GeometryStabilityResultWithBackend = GeometryStabilityResult & {
  backend: SimBackend
}
export type Universe3DResultWithBackend = Universe3DResult & { backend: SimBackend }
export type UniverseSliceResultWithBackend = UniverseSliceResult & {
  backend: SimBackend
}
export type FactorizationSearchResultWithBackend = FactorizationSearchResult & {
  backend: SimBackend
}
export type FactorizationRefinementResultWithBackend = FactorizationRefinementResult & {
  backend: SimBackend
}
export type FalsificationBatteryResultWithBackend = FalsificationBatteryResult & {
  backend: SimBackend
}

export async function runEmergenceAsync(
  config: RunConfig,
): Promise<RunResultWithBackend> {
  return runWasm('emergence', config)
}

export async function runSpacetimeAsync(
  config: SpacetimeRunConfig,
): Promise<SpacetimeResultWithBackend> {
  return runWasm('spacetime', config)
}

export async function runLightConeCompareAsync(
  config: SpacetimeRunConfig,
): Promise<LightConeCompareResultWithBackend> {
  return runWasm('lightConeCompare', config)
}

export async function runSpacetime2DAsync(
  config: Spacetime2DConfig,
): Promise<SpacetimeResultWithBackend> {
  return runWasm('spacetime2d', config)
}

export async function runHolographyAsync(
  config: HolographyRunConfig,
): Promise<HolographyRunResultWithBackend> {
  return runWasm('holography', config)
}

export async function runRtMassAsync(
  config: RtMassRunConfig,
): Promise<RtMassRunResultWithBackend> {
  return runWasm('rtMass', config)
}

export async function runRefinementQuenchAsync(
  config: RefinementQuenchConfig,
): Promise<RefinementQuenchResultWithBackend> {
  return runWasm('refinementQuench', config)
}

export async function runRefinementNCompareAsync(
  config: RefinementNCompareConfig,
): Promise<RefinementNCompareResultWithBackend> {
  return runWasm('refinementNCompare', config)
}

export async function runAdaptiveRefinementAsync(
  config: AdaptiveRefinementConfig,
): Promise<AdaptiveRefinementResultWithBackend> {
  return runWasm('adaptiveRefinement', config)
}

export type PredictiveRefinementResultWithBackend = PredictiveRefinementResult & {
  backend: SimBackend
}

export async function runPredictiveRefinementAsync(
  config: AdaptiveRefinementConfig,
): Promise<PredictiveRefinementResultWithBackend> {
  return runWasm('predictiveRefinement', config)
}

export type RtQuenchResultWithBackend = RtQuenchResult & { backend: SimBackend }

export async function runRtQuenchAsync(
  config: RtQuenchConfig,
): Promise<RtQuenchResultWithBackend> {
  return runWasm('rtQuench', config)
}

export type CurvatureProxyQuenchResultWithBackend = CurvatureProxyQuenchResult & {
  backend: SimBackend
}

export async function runCurvatureProxyQuenchAsync(
  config: CurvatureProxyQuenchConfig,
): Promise<CurvatureProxyQuenchResultWithBackend> {
  return runWasm('curvatureProxyQuench', config)
}

export async function runRelationalTimeAsync(
  config: RelationalTimeConfig,
): Promise<DualClockResultWithBackend> {
  return runWasm('relationalTime', config)
}

export async function runMultiClockAsync(
  config: MultiClockConfig,
): Promise<MultiClockResultWithBackend> {
  return runWasm('multiClock', config)
}

export type SimultaneityResultWithBackend = SimultaneityResult & { backend: SimBackend }

export async function runModularDualClockAsync(
  config: ModularDualClockConfig,
): Promise<ModularDualClockResultWithBackend> {
  return runWasm('modularDualClock', config)
}

export async function runSimultaneityAsync(
  config: SimultaneityConfig,
): Promise<SimultaneityResultWithBackend> {
  return runWasm('simultaneity', config)
}

export type BoostInvarianceResultWithBackend = BoostInvarianceResult & { backend: SimBackend }

export async function runLorentzScalingAsync(): Promise<LorentzScalingResult & { backend: SimBackend }> {
  return runWasm('lorentzScaling', {})
}

export async function runDispersionAsync(
  config: DispersionConfig,
): Promise<DispersionResult & { backend: SimBackend }> {
  return runWasm('dispersion', config)
}

export async function runBoostInvarianceAsync(
  config: BoostInvarianceConfig,
): Promise<BoostInvarianceResultWithBackend> {
  return runWasm('boostInvariance', config)
}

export async function runScatteringAsync(
  config: ScatteringConfig,
): Promise<ScatteringResultWithBackend> {
  return runWasm('scattering', config)
}

export async function runDecoherenceQuenchAsync(
  config: DecoherenceQuenchConfig,
): Promise<DecoherenceQuenchResultWithBackend> {
  return runWasm('decoherenceQuench', config)
}

export async function runExcitationSubspaceAsync(
  config: ExcitationSubspaceConfig,
): Promise<ExcitationSubspaceResultWithBackend> {
  return runWasm('excitationSubspace', config)
}

export type StabilizerSearchResultWithBackend = StabilizerSearchResult & { backend: SimBackend }

export async function runStabilizerSearchAsync(
  config: StabilizerSearchConfig,
): Promise<StabilizerSearchResultWithBackend> {
  return runWasm('stabilizerSearch', config)
}

export async function runParticleStabilityAsync(
  config: ParticleStabilityConfig,
): Promise<ParticleStabilityResult & { backend: SimBackend }> {
  return runWasm('particleStability', config)
}

export async function runBranchBornAsync(
  config: BranchBornConfig,
): Promise<BranchBornResult & { backend: SimBackend }> {
  return runWasm('branchBorn', config)
}

export async function runEftDimensionAsync(
  config: EftDimensionConfig,
): Promise<EftDimensionResult & { backend: SimBackend }> {
  return runWasm('eftDimension', config)
}

export async function runGeometryStabilityAsync(
  config: GeometryStabilityConfig,
): Promise<GeometryStabilityResultWithBackend> {
  return runWasm('geometryStability', config)
}

export async function runUniverse3DAsync(
  config: Universe3DConfig,
): Promise<Universe3DResultWithBackend> {
  return runWasm('universe3d', config)
}

export async function runUniverseSliceAsync(
  config: UniverseSliceConfig,
): Promise<UniverseSliceResultWithBackend> {
  return runWasm('universeSlice', config)
}

export async function runFactorizationSearchAsync(
  config: FactorizationSearchConfig,
): Promise<FactorizationSearchResultWithBackend> {
  warnIfExpensiveFactorizationSearch(config, 'wasm worker')
  return runWasm('factorizationSearch', config)
}

export async function runFactorizationRefinementStudyAsync(
  config: FactorizationRefinementConfig,
): Promise<FactorizationRefinementResultWithBackend> {
  warnIfExpensiveFactorizationRefinement(config, 'wasm worker')
  return runWasm('factorizationRefinement', config)
}

export async function runFalsificationBatteryAsync(): Promise<FalsificationBatteryResultWithBackend> {
  return runWasm('falsificationBattery', {})
}

export { probeWasmBackend } from './wasm/client.ts'
