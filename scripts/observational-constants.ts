/**
 * External literature caps for Phase 8 observational bridge.
 * Dimensionless proxies only — not a claim that sim units match lab units.
 */

export interface LiteratureBound {
  id: string
  label: string
  /** Representative dimensionless cap (order of magnitude). */
  value: number
  source: string
  note: string
}

/** Linear-in-energy LIV: δc/c ≲ δ_EE × (E/E_Planck) — use δ_EE as proxy cap. */
export const LIV_BOUNDS: LiteratureBound[] = [
  {
    id: 'grb-fermi',
    label: 'GRB time-of-flight (Fermi GBM)',
    value: 1e-15,
    source: 'Vasileiou et al., Phys. Rev. D 87, 122001 (2013); GRB 090510',
    note: 'Order-of-magnitude linear LIV coefficient δ_EE; sim ε is cardinal speed CoV, not δ_EE.',
  },
  {
    id: 'grb-tev',
    label: 'TeV GRB 221009A (dispersion)',
    value: 1e-17,
    source: 'LHAASO / Fermi-GBM joint analyses (2023)',
    note: 'Stricter time-of-flight bound at highest observed energies.',
  },
  {
    id: 'crab-synchrotron',
    label: 'Crab nebula synchrotron (electrons)',
    value: 1e-8,
    source: 'Kostelecký & Russell, Rev. Mod. Phys. 83, 11 (2011) — SME overview',
    note: 'Electron LIV; often quoted O(10⁻⁸–10⁻¹⁰) for quadratic terms.',
  },
]

/** Holographic / CFT reference points for RT slope calibration. */
export const HOLOGRAPHY_REFERENCES = {
  adsCftRtSlope: 1.0,
  adsCftNote:
    'Vacuum RT geodesic in AdS₃/CFT₂: S_A ∝ cut length at leading order (slope ≈ 1 in our discrete fit).',
  strongSubregionNote:
    'Excited-state or strong-subregion RT violations appear as slope deficit or S_A / (½ boundary MI) ≠ 1.',
  areaLawGroundNote: '1D gapped ground states satisfy area law generically — not AdS/CFT-specific.',
} as const

/** Lab-scale noise / code thresholds (order of magnitude). */
export const IR_NOISE_FLOORS: LiteratureBound[] = [
  {
    id: 'surface-code',
    label: 'Surface-code logical error (per cycle)',
    value: 0.01,
    source: 'Fowler et al., Phys. Rev. A 86, 032324 (2012) — ~1% threshold order',
    note: 'Compare to 1 − codeRate as effective redundancy gap per window.',
  },
  {
    id: 'superconducting-gate',
    label: 'Superconducting two-qubit gate fidelity',
    value: 0.001,
    source: 'IBM / Google benchmark papers (~99.9% fidelity)',
    note: 'Branch stabilizer sharpening margin vs mixed state.',
  },
]
