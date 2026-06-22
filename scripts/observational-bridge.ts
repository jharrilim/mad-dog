/**
 * Phase 8 observational bridge — orchestrates all export scripts.
 * Run: npm run bench:observational
 */

import { runObservationalLorentz } from './observational-lorentz.ts'
import { runObservationalHolography } from './observational-holography.ts'
import { runObservationalTime } from './observational-time.ts'
import { runObservationalIr } from './observational-ir.ts'

console.warn('[mad-dog observational] Phase-8 bridge: Lorentz, holography, time, IR subspace.')

const lorentz = runObservationalLorentz()
const holography = runObservationalHolography()
const time = runObservationalTime()
const ir = runObservationalIr()

console.log('\n=== Summary ===')
console.log(`  Lorentz: effective ε=${lorentz.epsilonEffective.toFixed(4)}`)
console.log(`  Holography: ${holography.exportPath}`)
console.log(`  Time: ${time.exportPath}`)
console.log(`  IR: ${ir.exportPath}`)
console.log('\nObservational bridge export complete.')
