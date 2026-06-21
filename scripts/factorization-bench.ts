/**
 * Factorization recovery benchmarks (WASM-only).
 * Run: npm run bench:factorization
 */

import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import {
  initSync,
  run_factorization_search_json,
} from '../src/sim/wasm/pkg/mad_dog_sim.js'
import { warnIfExpensiveFactorizationSearch } from '../src/sim/factorization-warnings.ts'
import type { FactorizationSearchConfig } from '../src/sim/types.ts'

const __dirname = dirname(fileURLToPath(import.meta.url))
initSync({ module: readFileSync(join(__dirname, '../src/sim/wasm/pkg/mad_dog_sim_bg.wasm')) })

const cases: { label: string; config: FactorizationSearchConfig; expectRecover?: boolean }[] = [
  { label: 'shuffled n=6', config: { kind: 'shuffled_chain', n: 6, field: 1.5, seed: 4242, topK: 3 }, expectRecover: true },
  { label: 'shuffled n=8', config: { kind: 'shuffled_chain', n: 8, field: 1.5, seed: 100, topK: 3 }, expectRecover: true },
  {
    label: 'shuffled n=10',
    config: { kind: 'shuffled_chain', n: 10, field: 1.5, seed: 200, topK: 3, searchMethod: 'annealing' },
  },
  { label: 'random n=6', config: { kind: 'random', n: 6, field: 1.5, seed: 7, topK: 3 } },
  {
    label: 'spectrum n=6',
    config: { kind: 'shuffled_chain', n: 6, field: 1.5, seed: 4242, topK: 3, inputMode: 'spectrum', eigenstateCount: 3 },
  },
  {
    label: 'grid vs line n=9',
    config: {
      kind: 'shuffled_grid',
      n: 9,
      rows: 3,
      cols: 3,
      field: 1.5,
      seed: 4242,
      topK: 3,
      graphKind: 'line',
      searchMethod: 'exact',
    },
    expectRecover: true,
  },
]

let failures = 0

console.warn(
  '[mad-dog factorization] bench runs WASM only; n=8 exact and n=10 annealing may take minutes.',
)

for (const { label, config, expectRecover } of cases) {
  warnIfExpensiveFactorizationSearch(config, label)
  const t0 = performance.now()
  const wasm = JSON.parse(run_factorization_search_json(JSON.stringify(config)))
  const elapsedMs = performance.now() - t0

  const recoverOk = expectRecover === undefined || wasm.recoveredIdentity === expectRecover
  const scoreOk = Number.isFinite(wasm.best.score) && wasm.best.score > 0
  if (!recoverOk || !scoreOk) failures++

  console.log(`\n${label} (${elapsedMs.toFixed(0)} ms):`)
  console.log(`  recovered=${wasm.recoveredIdentity}${expectRecover !== undefined ? (recoverOk ? ' OK' : ' FAIL') : ''}`)
  console.log(`  best score=${wasm.best.score.toFixed(3)}  locality=${(wasm.best.localityFraction * 100).toFixed(0)}%  method=${wasm.searchMethod}`)
}

console.log(failures === 0 ? '\nAll factorization benchmarks passed.' : `\n${failures} case(s) failed.`)
process.exit(failures === 0 ? 0 : 1)
