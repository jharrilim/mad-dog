import type { SimBackend } from './types'

export type { SimBackend }

let worker: Worker | null = null
let nextId = 1
const pending = new Map<
  number,
  { resolve: (r: unknown) => void; reject: (e: Error) => void }
>()

let wasmAvailable: boolean | null = null

function getWorker(): Worker {
  if (!worker) {
    worker = new Worker(new URL('./simWorker.ts', import.meta.url), {
      type: 'module',
    })
    worker.onmessage = (event: MessageEvent<{
      id: number
      ok: boolean
      result?: unknown
      error?: string
    }>) => {
      const msg = event.data
      const entry = pending.get(msg.id)
      if (!entry) return
      pending.delete(msg.id)
      if (msg.ok) entry.resolve(msg.result)
      else entry.reject(new Error(msg.error ?? 'WASM worker error'))
    }
    worker.onerror = (event) => {
      for (const [, entry] of pending) {
        entry.reject(new Error(event.message ?? 'WASM worker failed'))
      }
      pending.clear()
      wasmAvailable = false
    }
  }
  return worker
}

export type WasmMethod =
  | 'emergence'
  | 'spacetime'
  | 'spacetime2d'
  | 'holography'
  | 'rtMass'

export function callWasm<T>(method: WasmMethod, config: unknown): Promise<T> {
  const id = nextId++
  const w = getWorker()
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve: resolve as (r: unknown) => void, reject })
    w.postMessage({ id, method, config })
  })
}

export async function probeWasmBackend(): Promise<boolean> {
  if (wasmAvailable !== null) return wasmAvailable
  try {
    await callWasm('emergence', {
      kind: 'chain',
      n: 4,
      rows: 3,
      cols: 3,
      field: 1.5,
      seed: 1,
    })
    wasmAvailable = true
  } catch {
    wasmAvailable = false
  }
  return wasmAvailable
}

export function withBackend<T extends object>(
  result: T,
  backend: SimBackend,
): T & { backend: SimBackend } {
  return { ...result, backend }
}

export function terminateWasmWorker(): void {
  worker?.terminate()
  worker = null
  wasmAvailable = null
  pending.clear()
}

// Convenience wrappers
import type { RunConfig, RunResult } from '../runner'

export type RunResultWithBackend = RunResult & { backend: SimBackend }

export function runEmergenceWasm(
  config: RunConfig,
): Promise<RunResultWithBackend> {
  return callWasm<RunResultWithBackend>('emergence', config)
}
