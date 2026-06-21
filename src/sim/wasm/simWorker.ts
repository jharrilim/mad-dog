import init, {
  run_emergence_json,
  run_spacetime_json,
  run_spacetime_2d_json,
  run_holography_json,
  run_rt_mass_json,
  run_refinement_quench_json,
  run_refinement_n_compare_json,
  run_relational_time_json,
  run_universe_3d_json,
  run_universe_slice_json,
  run_factorization_search_json,
} from './pkg/mad_dog_sim.js'
import wasmUrl from './pkg/mad_dog_sim_bg.wasm?url'

export type WasmMethod =
  | 'emergence'
  | 'spacetime'
  | 'spacetime2d'
  | 'holography'
  | 'rtMass'
  | 'refinementQuench'
  | 'refinementNCompare'
  | 'relationalTime'
  | 'universe3d'
  | 'universeSlice'
  | 'factorizationSearch'

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
  spacetime2d: run_spacetime_2d_json,
  holography: run_holography_json,
  rtMass: run_rt_mass_json,
  refinementQuench: run_refinement_quench_json,
  refinementNCompare: run_refinement_n_compare_json,
  relationalTime: run_relational_time_json,
  universe3d: run_universe_3d_json,
  universeSlice: run_universe_slice_json,
  factorizationSearch: run_factorization_search_json,
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
