/**
 * Adaptive refinement sweep — when does split trigger and does n+Δ help?
 * Run: npm run sweep:refinement
 *
 * CLI: node scripts/refinement-adaptive.ts [--n 10] [--steps 18] [--delta 2]
 */

import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import {
  initSync,
  run_adaptive_refinement_json,
} from '../src/sim/wasm/pkg/mad_dog_sim.js'

const __dirname = dirname(fileURLToPath(import.meta.url))
initSync({ module: readFileSync(join(__dirname, '../src/sim/wasm/pkg/mad_dog_sim_bg.wasm')) })

function parseArgs(argv: string[]) {
  let n = 10
  let steps = 18
  let delta = 2
  let field = 1.5
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i]
    if (a === '--n' && argv[i + 1]) n = Number(argv[++i])
    else if (a === '--steps' && argv[i + 1]) steps = Number(argv[++i])
    else if (a === '--delta' && argv[i + 1]) delta = Number(argv[++i])
    else if (a === '--h' && argv[i + 1]) field = Number(argv[++i])
  }
  return { n, steps, delta, field }
}

const { n, steps, delta, field } = parseArgs(process.argv.slice(2))

console.log(`Adaptive refinement sweep: n=${n}, steps=${steps}, deltaN=${delta}, h=${field}\n`)
console.log(' step  trigger  peakP   preP   postP  accept  splitSite')
console.log(' ----  -------  ------  -----  ------  ------  ---------')

for (let s = 8; s <= steps; s += 2) {
  const r = JSON.parse(
    run_adaptive_refinement_json(
      JSON.stringify({ n, field, dt: 0.2, steps: s, seed: 7711, deltaN: delta }),
    ),
  )
  const ev = r.splitEvent
  const trig = ev ? String(ev.triggerStep).padStart(7) : '     —'
  const preP = ev ? ev.pre.pressure.toFixed(3).padStart(6) : '     —'
  const postP = ev ? ev.post.pressure.toFixed(3).padStart(6) : '     —'
  const accept = ev ? (ev.accepted ? '   yes' : '    no') : '     —'
  const site = ev ? String(ev.suggestedSplitSite).padStart(9) : '        —'
  console.log(
    ` ${String(s).padStart(4)}  ${trig}  ${r.peakPressure.toFixed(3).padStart(6)}  ${preP}  ${postP}  ${accept}  ${site}`,
  )
}

const full = JSON.parse(
  run_adaptive_refinement_json(
    JSON.stringify({ n, field, dt: 0.2, steps, seed: 7711, deltaN: delta }),
  ),
)
console.log('\nFull run summary:')
if (full.splitEvent) {
  const e = full.splitEvent
  console.log(
    `  trigger @ step ${e.triggerStep}, accepted=${e.accepted}, Δpressure=${e.pressureDelta.toFixed(3)}`,
  )
} else {
  console.log('  no split trigger in window')
}
