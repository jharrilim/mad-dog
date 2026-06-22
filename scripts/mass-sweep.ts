/**
 * RT slope vs excitation-density sweep (WASM-only).
 * Run: npm run sweep:mass
 */

import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import {
  initSync,
  run_rt_mass_json,
} from '../src/sim/wasm/pkg/mad_dog_sim.js'

const __dirname = dirname(fileURLToPath(import.meta.url))
initSync({ module: readFileSync(join(__dirname, '../src/sim/wasm/pkg/mad_dog_sim_bg.wasm')) })

const n = 10
const field = 1.5
const strength = 1.0
const seed = 7

const result = JSON.parse(
  run_rt_mass_json(
    JSON.stringify({
      n,
      field,
      strength,
      seed,
      densitySweep: true,
      maxMassCount: 5,
    }),
  ),
)

const sweep = result.report.densitySweep as
  | { count: number; density: number; slope: number; r2: number; deltaSlope: number }[]
  | undefined

if (!sweep?.length) {
  console.error('FAIL: densitySweep missing from report')
  process.exit(1)
}

const vacuum = result.report.vacuum.rtSlope

console.warn('[mad-dog mass] RT slope vs evenly spaced excitation count (fixed strength).')
console.log(`\nchain n=${n} h=${field} strength=${strength} vacuum slope=${vacuum.toFixed(3)}\n`)
console.log(' count | density |  slope |    R² | Δslope')
console.log('-------|---------|--------|-------|-------')

let failures = 0
for (const p of sweep) {
  console.log(
    ` ${String(p.count).padStart(5)} | ${p.density.toFixed(3).padStart(7)} | ${p.slope.toFixed(3).padStart(6)} | ${p.r2.toFixed(3)} | ${p.deltaSlope >= 0 ? '+' : ''}${p.deltaSlope.toFixed(3)}`,
  )
}

const first = sweep[0].deltaSlope
const last = sweep[sweep.length - 1].deltaSlope
if (last <= first + 0.05) {
  console.error('\nFAIL: Δslope did not grow meaningfully with density')
  failures++
}

const xs = sweep.map((p) => p.density)
const ys = sweep.map((p) => p.deltaSlope)
const xMean = xs.reduce((a, b) => a + b, 0) / xs.length
const yMean = ys.reduce((a, b) => a + b, 0) / ys.length
let num = 0
let den = 0
for (let i = 0; i < xs.length; i++) {
  num += (xs[i] - xMean) * (ys[i] - yMean)
  den += (xs[i] - xMean) ** 2
}
const slopeVsRho = den > 0 ? num / den : 0
console.log(`\nΔslope vs density linear fit: d(Δslope)/dρ ≈ ${slopeVsRho.toFixed(2)}`)

if (slopeVsRho <= 0) {
  console.error('FAIL: Δslope does not increase with excitation density')
  failures++
}

console.log(
  failures === 0
    ? '\nMass density sweep passed (Δslope grows with ρ).'
    : `\n${failures} check(s) failed.`,
)
process.exit(failures === 0 ? 0 : 1)
