/**
 * WASM-only numerical sanity checks.
 * Run: npm run ensure:wasm && node scripts/sim-check.ts
 */

import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import {
  initSync,
  run_emergence_json,
  run_spacetime_json,
  run_holography_json,
  run_rt_mass_json,
  run_relational_time_json,
  run_universe_3d_json,
  run_universe_slice_json,
  run_refinement_quench_json,
  run_refinement_n_compare_json,
} from '../src/sim/wasm/pkg/mad_dog_sim.js'

const __dirname = dirname(fileURLToPath(import.meta.url))
initSync({ module: readFileSync(join(__dirname, '../src/sim/wasm/pkg/mad_dog_sim_bg.wasm')) })

function approx(a: number, b: number, tol = 1e-6): string {
  return Math.abs(a - b) < tol ? 'OK' : `MISMATCH (got ${a}, want ${b})`
}

function summarize(label: string, emergentDim: number, eigenvalues: number[]) {
  const eig = eigenvalues.slice(0, 6).map((v) => v.toFixed(3))
  console.log(`\n${label}`)
  console.log('  emergent dimension :', emergentDim)
  console.log('  top eigenvalues    :', eig.join(', '))
}

let failures = 0
function fail(cond: boolean) {
  if (cond) failures++
}

// --- TFIM chain ---
{
  const wasm = JSON.parse(
    run_emergence_json(
      JSON.stringify({ kind: 'chain', n: 8, rows: 3, cols: 3, field: 1.5, seed: 12345 }),
    ),
  )
  console.log(`\n${wasm.label}: energy=${wasm.energy.toFixed(4)}`)
  summarize(wasm.label, wasm.report.mds.emergentDim, wasm.report.mds.eigenvalues)
  const mi = wasm.report.mi
  const nn = mi[0][1]
  const far = mi[0][7]
  console.log(
    '  MI(0:1) =',
    nn.toFixed(4),
    ' MI(0:7) =',
    far.toFixed(4),
    nn > far ? 'OK (neighbours more correlated)' : 'UNEXPECTED',
  )
  fail(wasm.report.mds.emergentDim !== 1)
}

// --- TFIM grid ---
{
  const wasm = JSON.parse(
    run_emergence_json(
      JSON.stringify({ kind: 'grid', n: 9, rows: 3, cols: 3, field: 1.5, seed: 12345 }),
    ),
  )
  console.log(`\n${wasm.label}: energy=${wasm.energy.toFixed(4)}`)
  summarize(wasm.label, wasm.report.mds.emergentDim, wasm.report.mds.eigenvalues)
  fail(wasm.report.mds.emergentDim !== 2)
}

// --- Random non-local ---
{
  const wasm = JSON.parse(
    run_emergence_json(
      JSON.stringify({ kind: 'random', n: 8, rows: 3, cols: 3, field: 1.5, seed: 777 }),
    ),
  )
  console.log(`\n${wasm.label}: energy=${wasm.energy.toFixed(4)}`)
  summarize(wasm.label, wasm.report.mds.emergentDim, wasm.report.mds.eigenvalues)
}

// --- Page-Wootters spacetime ---
{
  const result = JSON.parse(
    run_spacetime_json(JSON.stringify({ n: 9, field: 1.0, dt: 0.2, steps: 20, seed: 1 })),
  )
  console.log('\nPage-Wootters emergent spacetime (9-site chain, kick at center):')
  console.log(
    '  energy drift:',
    result.energyDrift.toExponential(2),
    result.energyDrift < 1e-3 ? 'OK (evolution ~unitary)' : 'HIGH',
  )
  fail(result.energyDrift >= 1e-3)

  const cone = result.lightCone
  if (cone) {
    console.log('  LR velocity from WASM:', cone.velocity.toFixed(3))
    fail(!(cone.velocity > 0))
  }

  const center = Math.floor(result.sites / 2)
  const arrival = Array.from({ length: result.sites }, () => Infinity)
  for (const slice of result.slices) {
    slice.signal.forEach((d: number, i: number) => {
      if (d > 0.05 && slice.t < arrival[i]) arrival[i] = slice.t
    })
  }
  let causal = true
  for (let i = center + 1; i < result.sites; i++) {
    if (arrival[i] < arrival[i - 1] - 1e-9) causal = false
  }
  for (let i = center - 1; i >= 0; i--) {
    if (arrival[i] < arrival[i + 1] - 1e-9) causal = false
  }
  console.log('  arrival delayed with distance (finite speed):', causal ? 'OK' : 'NO')
  fail(!causal)
}

// --- Holography / RT ---
{
  const report = JSON.parse(
    run_holography_json(JSON.stringify({ n: 10, field: 1.5, seed: 31337 })),
  ).report
  console.log('\nBaby Ryu-Takayanagi (10-site chain, h=1.5):')
  const mid = report.areaLaw[Math.floor(report.areaLaw.length / 2)]
  console.log(
    `  at |A|=${mid.size}: ground=${mid.sGround.toFixed(3)} << random=${mid.sRandom.toFixed(3)}`,
    mid.sGround < 0.6 * mid.sRandom ? 'OK (area << volume)' : 'UNEXPECTED',
  )
  console.log(
    `  RT fit slope=${report.rtSlope.toFixed(3)}, R²=${report.rtR2.toFixed(4)}`,
    report.rtR2 > 0.9 ? 'OK' : 'WEAK',
  )
  fail(report.rtR2 <= 0.9)
}

// --- Two clocks ---
{
  const edge = JSON.parse(
    run_relational_time_json(
      JSON.stringify({ n: 9, field: 1, dt: 0.2, steps: 40, clockSite: 0, physicalSlices: 15 }),
    ),
  )
  console.log('\nTwo-clock relational time:')
  console.log(
    `  edge clock sync R² = ${edge.syncR2.toFixed(4)}`,
    edge.syncR2 < 0.95 ? 'OK' : 'UNEXPECTED',
  )
  fail(edge.syncR2 >= 0.95)
}

// --- RT mass ---
{
  const r = JSON.parse(
    run_rt_mass_json(JSON.stringify({ n: 10, field: 1.5, strength: 1.0, seed: 7 })),
  ).report
  const delta = r.mass.rtSlope - r.vacuum.rtSlope
  console.log('\nRT slope under mass:')
  console.log(`  vacuum=${r.vacuum.rtSlope.toFixed(3)}, mass=${r.mass.rtSlope.toFixed(3)}, Δ=${delta.toFixed(3)}`)
  fail(delta <= 0.2)
}

// --- Cube emergence ---
{
  const wasm = JSON.parse(
    run_emergence_json(
      JSON.stringify({ kind: 'grid', n: 8, rows: 2, cols: 2, field: 1.5, seed: 42 }),
    ),
  )
  // 2x2x2 uses universe path; use universe3d for cube quench instead below
  summarize('2x2 grid slice', wasm.report.mds.emergentDim, wasm.report.mds.eigenvalues)
}

// --- 3+1 universe ---
{
  const uni = JSON.parse(
    run_universe_3d_json(JSON.stringify({ lx: 2, ly: 2, lz: 2, field: 1, dt: 0.25, steps: 12 })),
  )
  const slice = uni.spacetime.slices[0]
  const dim3 = slice.coords.every((c: number[]) => c.length >= 3)
  console.log('\n3+1 universe (2x2x2 cube quench):')
  console.log('  3D coords:', dim3 ? 'OK' : 'NO')
  console.log('  LR velocity:', uni.lightCone.velocity.toFixed(3))
  fail(!dim3 || uni.lightCone.velocity <= 0)

  const mid = JSON.parse(
    run_universe_slice_json(
      JSON.stringify({ lx: 2, ly: 2, lz: 3, field: 1.5, dt: 0.25, steps: 12, k: 3 }),
    ),
  )
  console.log('  2x2x3 quench k=3 emergentDim:', mid.report.mds.emergentDim)
  fail(mid.report.mds.emergentDim < 3)
}

// --- Refinement ---
{
  const quench = JSON.parse(
    run_refinement_quench_json(
      JSON.stringify({ n: 10, field: 1.5, dt: 0.2, steps: 16, seed: 7711 }),
    ),
  )
  const vacuum = quench.slices[0].diagnostics
  const late = quench.slices[quench.slices.length - 1].diagnostics
  console.log('\nRefinement quench:')
  console.log(`  vacuum pressure=${vacuum.pressure.toFixed(3)}, late=${late.pressure.toFixed(3)}`)
  console.log(`  decoupling lag=${quench.decouplingLag}`)
  fail(late.pressure <= vacuum.pressure)

  const cmp = JSON.parse(
    run_refinement_n_compare_json(
      JSON.stringify({ n: 10, deltaN: 2, field: 1.5, dt: 0.2, quenchStep: 12, seed: 7711 }),
    ),
  )
  console.log(
    `  n compare: ${cmp.small.pressure.toFixed(3)} → ${cmp.large.pressure.toFixed(3)}`,
    cmp.largerRelieves ? 'OK' : 'INCONCLUSIVE',
  )
}

console.log(failures === 0 ? '\nAll sim-check assertions passed.' : `\n${failures} assertion(s) failed.`)
process.exit(failures === 0 ? 0 : 1)
