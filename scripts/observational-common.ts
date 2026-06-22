import { mkdirSync, writeFileSync } from 'node:fs'
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import { initSync } from '../src/sim/wasm/pkg/mad_dog_sim.js'

const __dirname = dirname(fileURLToPath(import.meta.url))
const wasmPath = join(__dirname, '../src/sim/wasm/pkg/mad_dog_sim_bg.wasm')
const exportDir = join(__dirname, '../exports/observational')

let initialized = false

export function ensureWasm(): void {
  if (!initialized) {
    initSync({ module: readFileSync(wasmPath) })
    initialized = true
  }
}

export function writeExport(name: string, data: unknown): string {
  mkdirSync(exportDir, { recursive: true })
  const path = join(exportDir, `${name}.json`)
  writeFileSync(path, JSON.stringify(data, null, 2) + '\n')
  return path
}
