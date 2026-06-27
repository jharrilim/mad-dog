/**
 * Phase 12 scaling research probe (one-off).
 * Run: npm run build:wasm && node scripts/scaling-probe.ts
 */

import { readFileSync, writeFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import {
  initSync,
  run_factorization_search_json,
  run_locality_spectrum_json,
  run_lorentz_scaling_json,
  run_multi_clock_json,
} from '../src/sim/wasm/pkg/mad_dog_sim.js'
import type { FactorizationSearchConfig, FactorizationSearchResult } from '../src/sim/types.ts'

const __dirname = dirname(fileURLToPath(import.meta.url))
initSync({ module: readFileSync(join(__dirname, '../src/sim/wasm/pkg/mad_dog_sim_bg.wasm')) })

type Row = Record<string, string | number | boolean>

function runFactorization(label: string, config: FactorizationSearchConfig): Row {
  const t0 = performance.now()
  const r = JSON.parse(run_factorization_search_json(JSON.stringify(config))) as FactorizationSearchResult
  const ms = performance.now() - t0
  const u = r.uniqueness
  return {
    probe: label,
    n: r.n,
    recovered: r.recoveredIdentity,
    score: +r.best.score.toFixed(4),
    localityPct: +(r.best.localityFraction * 100).toFixed(1),
    miNn: +r.best.miNnRatio.toFixed(3),
    method: r.searchMethod,
    permDist: r.permMatchDistance ?? -1,
    classes: u?.equivalenceClassCount ?? -1,
    bestClassSize: u?.bestClassSize ?? -1,
    scoreGap: u ? +u.scoreGapToSecondClass.toFixed(4) : -1,
    trueInTopK: u?.trueInTopK ?? false,
    ms: +ms.toFixed(0),
  }
}

const rows: Row[] = []

console.log('=== Factorization spectrum scaling (shuffled_chain, h=1.5, seed=4242) ===')
for (const n of [4, 5, 6, 7, 8]) {
  rows.push(
    runFactorization(`spectrum_chain_n${n}`, {
      kind: 'shuffled_chain',
      n,
      field: 1.5,
      seed: 4242,
      topK: 5,
      inputMode: 'spectrum',
      eigenstateCount: n <= 4 ? 2 : 3,
      searchMethod: 'exact',
    }),
  )
}

console.log('=== Factorization pauli scaling (same seeds) ===')
for (const n of [4, 5, 6, 7, 8]) {
  rows.push(
    runFactorization(`pauli_chain_n${n}`, {
      kind: 'shuffled_chain',
      n,
      field: 1.5,
      seed: 4242,
      topK: 5,
      searchMethod: 'exact',
    }),
  )
}

console.log('=== Uniqueness: spectrum n=6 vs n=8 (seed=4242 / 100) ===')
rows.push(
  runFactorization('uniqueness_spectrum_n6', {
    kind: 'shuffled_chain',
    n: 6,
    field: 1.5,
    seed: 4242,
    topK: 5,
    inputMode: 'spectrum',
    eigenstateCount: 3,
    searchMethod: 'exact',
  }),
)
rows.push(
  runFactorization('uniqueness_spectrum_n8', {
    kind: 'shuffled_chain',
    n: 8,
    field: 1.5,
    seed: 100,
    topK: 5,
    inputMode: 'spectrum',
    eigenstateCount: 3,
    searchMethod: 'exact',
  }),
)

console.log('=== 2D spectrum (semi-blind: spectrum+Ĥ locality term, NOT AA-blind) ===')
rows.push(
  runFactorization('spectrum_grid_3x3', {
    kind: 'shuffled_grid',
    n: 9,
    rows: 3,
    cols: 3,
    field: 1.5,
    seed: 4242,
    topK: 5,
    inputMode: 'spectrum',
    eigenstateCount: 3,
    graphKind: 'grid',
    searchMethod: 'exact',
  }),
)
rows.push(
  runFactorization('spectrum_torus_2x2', {
    kind: 'shuffled_torus',
    n: 4,
    rows: 2,
    cols: 2,
    field: 1.5,
    seed: 4242,
    topK: 5,
    inputMode: 'spectrum',
    eigenstateCount: 2,
    graphKind: 'torus',
    searchMethod: 'exact',
  }),
)

console.log('=== Locality spectrum battery (AA, n=6 only) ===')
const aa = JSON.parse(run_locality_spectrum_json('{}')) as {
  total: number
  recovered: number
  recoveryRate: number
  pass: boolean
  cases: { model: string; recovered: boolean; score: number; miNnRatio: number }[]
}
rows.push({
  probe: 'AA_battery',
  n: 6,
  recovered: aa.pass,
  score: aa.recoveryRate,
  localityPct: aa.recovered,
  miNn: aa.total,
  method: 'AA',
  permDist: -1,
  classes: -1,
  bestClassSize: -1,
  scoreGap: -1,
  trueInTopK: aa.pass,
  ms: 0,
})

console.log('=== Lorentz scaling (L′) ===')
const lorentz = JSON.parse(run_lorentz_scaling_json('{}')) as {
  allPassed: boolean
  covSmall: number
  covLarge: number
  covImproves: boolean
  cases: { label: string; speedCv: number; passed: boolean }[]
}
for (const c of lorentz.cases) {
  rows.push({
    probe: `lorentz_${c.label}`,
    n: c.label.includes('4') ? 16 : 9,
    recovered: c.passed,
    score: +c.speedCv.toFixed(4),
    localityPct: lorentz.covImproves ? 1 : 0,
    miNn: lorentz.covSmall,
    method: 'Lprime',
    permDist: -1,
    classes: -1,
    bestClassSize: -1,
    scoreGap: lorentz.covLarge,
    trueInTopK: lorentz.allPassed,
    ms: 0,
  })
}

console.log('=== Multi-clock chain minPairwiseR2 ===')
for (const n of [7, 9, 11]) {
  const t0 = performance.now()
  const mc = JSON.parse(
    run_multi_clock_json(
      JSON.stringify({ kind: 'chain', n, field: 1, dt: 0.2, steps: 40, physicalSlices: 15 }),
    ),
  ) as { defectUniformR2: number; minPairwiseR2: number; inconsistentPairs: number }
  const pass = mc.defectUniformR2 > 0.95 && mc.minPairwiseR2 < 0.95
  rows.push({
    probe: `multiclock_chain_n${n}`,
    n,
    recovered: pass,
    score: +mc.minPairwiseR2.toFixed(4),
    localityPct: +(mc.defectUniformR2 * 100).toFixed(1),
    miNn: mc.inconsistentPairs,
    method: 'G',
    permDist: -1,
    classes: -1,
    bestClassSize: -1,
    scoreGap: -1,
    trueInTopK: pass,
    ms: +(performance.now() - t0).toFixed(0),
  })
}

const outPath = join(__dirname, '../exports/scaling-probe.json')
writeFileSync(outPath, JSON.stringify({ generatedAt: new Date().toISOString(), rows, aaCases: aa.cases, lorentz }, null, 2))

console.log('\n--- Results ---')
for (const r of rows) {
  console.log(
    `${String(r.probe).padEnd(28)} n=${String(r.n).padStart(2)} rec=${String(r.recovered).padEnd(5)} score=${r.score} loc=${r.localityPct}% gap=${r.scoreGap} ${r.ms}ms`,
  )
}
console.log(`\nWrote ${outPath}`)
console.log('\nNote: true AA-blind (spectrum_scrambled, no Ĥ) for 2D requires cargo test scaling_probe_blind_2d')
