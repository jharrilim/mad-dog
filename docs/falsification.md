# Falsification Criteria

What would **disprove** Mad-Dog-specific emergence claims — as opposed to generic quantum-information facts (area laws in 1D gapped systems, etc.).

## Generic vs specific

| Observation | Generic QI? | Mad-Dog-specific? |
|-------------|-------------|-------------------|
| Ground-state area law | Yes (gapped 1D) | No |
| RT slope ≈ 1 on ground | Approximate for special states | Partially |
| Emergent dim = lattice dim (TFIM chosen to match) | Circular if model known | Weak |
| RT slope ↑ under excitation | Expected if RT is class-specific | No (failure of identity, not curvature) |

## Automated battery

Run: `npm run check:falsification` or **Experiments → Falsification Tests**.

| ID | Claim tested | Pass criterion |
|----|--------------|----------------|
| **A** | Blind locality recovery | Shuffled chain n=8 recovers identity permutation |
| **A′** | Cross-graph structure | Open grid: line ≈ grid score; torus: line locality < 90% |
| **B** | MI geometry faithful | \|v_MI / v_lattice − 1\| < 0.25 |
| **C** | RT ratio on ground | std(S_A / ½ boundary MI) < 0.15 across intervals |
| **D** | Modular clocks desync | In-cone modular sync R² < 0.95 |
| **D′** | Modular and Z edge clocks desync from uniform Δt | minModUniformR² < 0.95 and minZEdgeUniformR² < 0.95 |
| **E** | Excitations propagate | Both worldlines move; min separation ≥ 1 during overlap |
| **F** | Refinement decoupling | Slope vs area diagnostic peaks diverge under quench |
| **G** | Multi-clock not globally consistent | defectUniformR² > 0.95 and minPairwiseR² < 0.95 |
| **G′** | Grid multi-clock not globally consistent | Same on 3×3 TFIM grid quench |
| **N** | Simultaneity surfaces bend | Defect vs edge clock: `bendDetected` on emergent foliation |
| **H** | Adaptive split relieves pressure | Split triggers and accepted on default quench |
| **I** | Branch excitations code-like | Sharpness gain > 1.03, window rank drops, branch overlap < 0.95 |
| **J** | Gauge-free MI geometry stable (chain) | Ordered phase mean Spearman ρ > 0.85 |
| **J′** | 3D gauge-free geometry (2×2×3 cube) | mean ρ > 0.85 and `dimStable` on quench |
| **K** | Ensemble blind locality recovery | Phase-1 model zoo: ≥80% recovery (spectrum; Pauli for Heisenberg/sparse) |
| **L** | Directional causal isotropy (Lorentz proxy v0) | Cardinal front-speed CoV < 0.25 on 3×3 grid quench |
| **L′** | Cardinal speed CoV improves with grid size | 3×3→4×4 CoV non-increasing and both < 0.25 |
| **O** | Dispersion ω(k) linear at small k | Wavepacket v_g CoV < 0.35 and ω∝k fit R² > 0.85 |
| **P** | Weak boost invariance | Uniform vs edge-clock light-cone speed relΔ < 0.20 |
| **Q** | Scattering exchange phase stable | Detrended post-interaction phase residual std < 0.55 |
| **R′** | RT slope deficit structured vs density under quench | `structuredDeviation` on defect quench |
| **F′** | Predictive RT warning precedes failure; late split recovers | leadTime > 0 and late split accepted |
| **C′** | Curvature proxy suite internally consistent | Geodesic deviation tracks density or cross-proxy |
| **I′** | Pauli stabilizer generators on branch subspace | ≥1 generator, distance ≥1, window scaling |
| **S′** | Defect more localized in ordered phase | `orderedLongerLived` on h=0.5 vs 2.5 |
| **T′** | Branch Born weights co-move with distinguishability | Env-entropy vs overlap ρ or strong imbalance ρ |
| **U′** | EFT DOF per site vs stabilizer code rate | `dofAgreement` within tolerance |
| **V′** | In-place tensor split relieves pressure | `inPlaceImproves` at peak-pressure trigger |
| **W′** | Holographic bound n_min finite with plateau | `nMin` Some and `boundScales` on chain sweep |
| **M** | Negative controls reject fake locality | `random` n=6 and scrambled-spectrum shuffled chain do not recover |

**K**, **L**, **L′**, **O**, **P**, **Q**, **R′**, **F′**, **C′**, **I′**, **S′**, **T′**, **U′**, **V′**, **W′**, and **M** target [roadmap.md](./roadmap.md) signatures **S1**, **S4**, **S5**, **S6–S8**, **S9**, and **S10**.

## Planned extensions

| ID | Signature | Target criterion (not yet in battery) |
|----|-----------|----------------------------------------|
| _(none — V′/W′ shipped)_ | | |

## Implementation

- Rust: [`wasm/mad-dog-sim/src/run_falsification.rs`](../wasm/mad-dog-sim/src/run_falsification.rs) → `run_falsification_battery_json`
- Ensemble: [`wasm/mad-dog-sim/src/run_factorization_ensemble.rs`](../wasm/mad-dog-sim/src/run_factorization_ensemble.rs) → `run_factorization_ensemble_json`
- CLI: `npm run check:falsification` → [`scripts/falsification-check.ts`](../scripts/falsification-check.ts)
- UI: [`FalsificationViz`](../src/components/FalsificationViz.tsx) (async WASM worker)

Thresholds are hand-tuned on TFIM demos; treat failures as research signals, not theorem violations.
