import {
  warnIfExpensiveFactorizationRefinement,
  warnIfExpensiveFactorizationSearch,
} from '../factorization-warnings.ts'
import init, {
  run_emergence_json,
  run_spacetime_json,
  run_light_cone_compare_json,
  run_spacetime_2d_json,
  run_holography_json,
  run_rt_mass_json,
  run_refinement_quench_json,
  run_refinement_n_compare_json,
  run_adaptive_refinement_json,
  run_predictive_refinement_json,
  run_rt_quench_json,
  run_curvature_proxy_quench_json,
  run_relational_time_json,
  run_multi_clock_json,
  run_modular_dual_clock_json,
  run_simultaneity_json,
  run_lorentz_scaling_json,
  run_dispersion_json,
  run_boost_invariance_json,
  run_scattering_json,
  run_decoherence_quench_json,
  run_excitation_subspace_json,
  run_geometry_stability_json,
  run_geometry_dim_sweep_json,
  run_universe_3d_json,
  run_universe_slice_json,
  run_factorization_search_json,
  run_factorization_refinement_json,
  run_factorization_ensemble_json,
  run_falsification_battery_json,
  run_stabilizer_search_json,
  run_particle_stability_json,
  run_branch_born_json,
  run_eft_dimension_json,
} from './pkg/mad_dog_sim.js'
import wasmUrl from './pkg/mad_dog_sim_bg.wasm?url'

export type WasmMethod =
  | 'emergence'
  | 'spacetime'
  | 'lightConeCompare'
  | 'spacetime2d'
  | 'holography'
  | 'rtMass'
  | 'refinementQuench'
  | 'refinementNCompare'
  | 'adaptiveRefinement'
  | 'predictiveRefinement'
  | 'rtQuench'
  | 'curvatureProxyQuench'
  | 'relationalTime'
  | 'multiClock'
  | 'modularDualClock'
  | 'simultaneity'
  | 'lorentzScaling'
  | 'dispersion'
  | 'boostInvariance'
  | 'scattering'
  | 'decoherenceQuench'
  | 'excitationSubspace'
  | 'geometryStability'
  | 'geometryDimSweep'
  | 'universe3d'
  | 'universeSlice'
  | 'factorizationSearch'
  | 'factorizationRefinement'
  | 'factorizationEnsemble'
  | 'falsificationBattery'
  | 'stabilizerSearch'
  | 'particleStability'
  | 'branchBorn'
  | 'eftDimension'

export type WorkerRequest = {
  id: number
  method: WasmMethod
  config: unknown
}

export type WorkerResponse =
  | { id: number; ok: true; result: unknown }
  | { id: number; ok: false; error: string }

const runners: Record<WasmMethod, (json: string) => string> = {
  emergence: run_emergence_json,
  spacetime: run_spacetime_json,
  lightConeCompare: run_light_cone_compare_json,
  spacetime2d: run_spacetime_2d_json,
  holography: run_holography_json,
  rtMass: run_rt_mass_json,
  refinementQuench: run_refinement_quench_json,
  refinementNCompare: run_refinement_n_compare_json,
  adaptiveRefinement: run_adaptive_refinement_json,
  predictiveRefinement: run_predictive_refinement_json,
  rtQuench: run_rt_quench_json,
  curvatureProxyQuench: run_curvature_proxy_quench_json,
  relationalTime: run_relational_time_json,
  multiClock: run_multi_clock_json,
  modularDualClock: run_modular_dual_clock_json,
  simultaneity: run_simultaneity_json,
  lorentzScaling: run_lorentz_scaling_json,
  dispersion: run_dispersion_json,
  boostInvariance: run_boost_invariance_json,
  scattering: run_scattering_json,
  decoherenceQuench: run_decoherence_quench_json,
  excitationSubspace: run_excitation_subspace_json,
  geometryStability: run_geometry_stability_json,
  geometryDimSweep: run_geometry_dim_sweep_json,
  universe3d: run_universe_3d_json,
  universeSlice: run_universe_slice_json,
  factorizationSearch: run_factorization_search_json,
  factorizationRefinement: run_factorization_refinement_json,
  factorizationEnsemble: run_factorization_ensemble_json,
  falsificationBattery: run_falsification_battery_json,
  stabilizerSearch: run_stabilizer_search_json,
  particleStability: run_particle_stability_json,
  branchBorn: run_branch_born_json,
  eftDimension: run_eft_dimension_json,
}

let initPromise: Promise<void> | null = null

function ensureWasm(): Promise<void> {
  if (!initPromise) {
    initPromise = init(wasmUrl).then(() => undefined)
  }
  return initPromise
}

self.onmessage = async (event: MessageEvent<WorkerRequest>) => {
  const { id, method, config } = event.data
  try {
    await ensureWasm()
    const run = runners[method]
    if (!run) throw new Error(`Unknown WASM method: ${method}`)
    if (method === 'factorizationSearch') {
      warnIfExpensiveFactorizationSearch(
        config as Parameters<typeof warnIfExpensiveFactorizationSearch>[0],
        'wasm worker',
      )
    } else if (method === 'factorizationRefinement') {
      warnIfExpensiveFactorizationRefinement(
        config as Parameters<typeof warnIfExpensiveFactorizationRefinement>[0],
        'wasm worker',
      )
    }
    const json = run(JSON.stringify(config))
    const result = JSON.parse(json) as unknown
    const response: WorkerResponse = { id, ok: true, result }
    self.postMessage(response)
  } catch (err) {
    const response: WorkerResponse = {
      id,
      ok: false,
      error: err instanceof Error ? err.message : String(err),
    }
    self.postMessage(response)
  }
}
