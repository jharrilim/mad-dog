/**
 * Worker thread: one scattering case via WASM.
 */

import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import { parentPort, workerData } from 'node:worker_threads'
import { initSync, run_scattering_json } from '../src/sim/wasm/pkg/mad_dog_sim.js'

const __dirname = dirname(fileURLToPath(import.meta.url))
initSync({ module: readFileSync(join(__dirname, '../src/sim/wasm/pkg/mad_dog_sim_bg.wasm')) })

const t0 = performance.now()
const result = JSON.parse(run_scattering_json(JSON.stringify({ ...workerData, lite: true })))
parentPort!.postMessage({
  n: workerData.n,
  field: workerData.field,
  crossed: result.crossed,
  minSep: result.minSeparation,
  elapsedMs: performance.now() - t0,
})
