/**
 * Falsification battery CLI (WASM-only).
 * Run: npm run check:falsification
 */

import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import { initSync, run_falsification_battery_json } from '../src/sim/wasm/pkg/mad_dog_sim.js'
import type { FalsificationBatteryResult } from '../src/sim/types.ts'

const __dirname = dirname(fileURLToPath(import.meta.url))
initSync({ module: readFileSync(join(__dirname, '../src/sim/wasm/pkg/mad_dog_sim_bg.wasm')) })

console.warn(
  '[mad-dog factorization] Falsification battery includes exact factorization searches, ' +
    'Phase-1 ensemble (K), and negative controls (M) — may take several minutes.',
)

const result = JSON.parse(run_falsification_battery_json('{}')) as FalsificationBatteryResult
let failures = 0

console.log('\nFalsification battery (WASM):\n')
for (const t of result.tests) {
  const status = t.passed ? 'PASS' : 'FAIL'
  if (!t.passed) failures++
  console.log(`  [${t.id}] ${status} — ${t.name}`)
  console.log(`       ${t.detail}`)
}

console.log(
  `\n${result.passed}/${result.total} passed (${result.elapsedMs.toFixed(0)} ms, backend=${result.backend})`,
)
process.exit(failures === 0 ? 0 : 1)
