/**
 * Async sim drivers: WASM worker when available, TypeScript reference otherwise.
 */

import {
  runEmergence,
  runHolography,
  runRtMass,
  runSpacetime,
  runSpacetime2D,
  runRefinementQuench,
  runRefinementNCompare,
  runRelationalTime,
  runUniverse3D,
  runUniverseSlice,
  runFactorizationSearch,
  type HolographyRunConfig,
  type HolographyRunResult,
  type RunConfig,
  type RunResult,
  type RtMassRunConfig,
  type RtMassRunResult,
  type RefinementQuenchConfig,
  type RefinementQuenchResult,
  type RefinementNCompareConfig,
  type RefinementNCompareResult,
  type Spacetime2DConfig,
  type SpacetimeRunConfig,
  type RelationalTimeConfig,
  type Universe3DConfig,
  type Universe3DResult,
  type UniverseSliceConfig,
  type UniverseSliceResult,
  type FactorizationSearchConfig,
  type FactorizationSearchResult,
} from './runner'
import type { SpacetimeResult } from './spacetime'
import type { DualClockResult } from './relational-time'
import { callWasm, withBackend, type SimBackend } from './wasm/client'

let preferWasm = true

async function runWithFallback<T extends { elapsedMs: number }>(
  method: Parameters<typeof callWasm>[0],
  config: unknown,
  fallback: () => T,
): Promise<T & { backend: SimBackend }> {
  if (preferWasm) {
    try {
      const result = await callWasm<T>(method, config)
      return withBackend(result, 'wasm')
    } catch {
      preferWasm = false
    }
  }
  const start = performance.now()
  const result = fallback()
  return {
    ...result,
    backend: 'typescript',
    elapsedMs: result.elapsedMs || performance.now() - start,
  }
}

export type RunResultWithBackend = RunResult & { backend: SimBackend }
export type SpacetimeResultWithBackend = SpacetimeResult & { backend: SimBackend }
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
export type DualClockResultWithBackend = DualClockResult & { backend: SimBackend }
export type Universe3DResultWithBackend = Universe3DResult & { backend: SimBackend }
export type UniverseSliceResultWithBackend = UniverseSliceResult & {
  backend: SimBackend
}
export type FactorizationSearchResultWithBackend = FactorizationSearchResult & {
  backend: SimBackend
}

export async function runEmergenceAsync(
  config: RunConfig,
): Promise<RunResultWithBackend> {
  return runWithFallback('emergence', config, () => runEmergence(config))
}

export async function runSpacetimeAsync(
  config: SpacetimeRunConfig,
): Promise<SpacetimeResultWithBackend> {
  return runWithFallback('spacetime', config, () => runSpacetime(config))
}

export async function runSpacetime2DAsync(
  config: Spacetime2DConfig,
): Promise<SpacetimeResultWithBackend> {
  return runWithFallback('spacetime2d', config, () => runSpacetime2D(config))
}

export async function runHolographyAsync(
  config: HolographyRunConfig,
): Promise<HolographyRunResultWithBackend> {
  return runWithFallback('holography', config, () => runHolography(config))
}

export async function runRtMassAsync(
  config: RtMassRunConfig,
): Promise<RtMassRunResultWithBackend> {
  return runWithFallback('rtMass', config, () => runRtMass(config))
}

export async function runRefinementQuenchAsync(
  config: RefinementQuenchConfig,
): Promise<RefinementQuenchResultWithBackend> {
  return runWithFallback('refinementQuench', config, () =>
    runRefinementQuench(config),
  )
}

export async function runRefinementNCompareAsync(
  config: RefinementNCompareConfig,
): Promise<RefinementNCompareResultWithBackend> {
  return runWithFallback('refinementNCompare', config, () =>
    runRefinementNCompare(config),
  )
}

export async function runRelationalTimeAsync(
  config: RelationalTimeConfig,
): Promise<DualClockResultWithBackend> {
  return runWithFallback('relationalTime', config, () => runRelationalTime(config))
}

export async function runUniverse3DAsync(
  config: Universe3DConfig,
): Promise<Universe3DResultWithBackend> {
  return runWithFallback('universe3d', config, () => runUniverse3D(config))
}

export async function runUniverseSliceAsync(
  config: UniverseSliceConfig,
): Promise<UniverseSliceResultWithBackend> {
  return runWithFallback('universeSlice', config, () => runUniverseSlice(config))
}

export async function runFactorizationSearchAsync(
  config: FactorizationSearchConfig,
): Promise<FactorizationSearchResultWithBackend> {
  return runWithFallback('factorizationSearch', config, () =>
    runFactorizationSearch(config),
  )
}

export { probeWasmBackend } from './wasm/client'
