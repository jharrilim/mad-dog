/**
 * Phase 12 scaling battery (WASM-only, fast regression).
 * Run: npm run check:scaling
 */

import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import { initSync, run_scaling_battery_json } from '../src/sim/wasm/pkg/mad_dog_sim.js'

const __dirname = dirname(fileURLToPath(import.meta.url))
initSync({ module: readFileSync(join(__dirname, '../src/sim/wasm/pkg/mad_dog_sim_bg.wasm')) })

type ScalingCheck = { id: string; pass: boolean; detail: string }
type ScalingBatteryResult = { pass: boolean; checks: ScalingCheck[]; elapsedMs: number }

const result = JSON.parse(run_scaling_battery_json('{}')) as ScalingBatteryResult
let failures = 0

console.log('\nScaling battery (WASM):\n')
for (const c of result.checks) {
  const status = c.pass ? 'PASS' : 'FAIL'
  if (!c.pass) failures++
  console.log(`  [${c.id}] ${status} — ${c.detail}`)
}

console.log(
  `\n${result.checks.filter((c) => c.pass).length}/${result.checks.length} passed (${result.elapsedMs.toFixed(0)} ms)`,
)
process.exit(failures === 0 ? 0 : 1)
