/**
 * Async sim drivers: WASM worker when available, TypeScript reference otherwise.
 */

import {
  runEmergence,
  runHolography,
  runRtMass,
  runSpacetime,
  runSpacetime2D,
  type HolographyRunConfig,
  type HolographyRunResult,
  type RunConfig,
  type RunResult,
  type RtMassRunConfig,
  type RtMassRunResult,
  type Spacetime2DConfig,
  type SpacetimeRunConfig,
} from './runner'
import type { SpacetimeResult } from './spacetime'
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

export { probeWasmBackend } from './wasm/client'
