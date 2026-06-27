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
| **AD** | Exchange phase shifts at cone overlap | `overlapDetected` and \|interactionPhaseShift\| > 0.05 and finite `separationTimeDelay` on default chain demo |
| **AE** | Two-defect scattering on 2D TFIM grid | 3×3 grid: `bothMoved` and Manhattan `minSeparation` < initial defect separation |
| **AF** | Effective mass from dispersion ω(k) on scattering chain | Chain: `effectiveMass` > 0.01 and `dispersionVelocityMean` > 0.01 (ω² = m² + v²k² fit) |
| **R′** | RT slope deficit structured vs density under quench *(prototype diagnostic)* | `structuredDeviation` on hold-out `(n, field, seed)` grid (3/3); calibration `n=10, h=1.5, seed=7711` informational |
| **F′** | Predictive RT warning precedes failure; late split recovers *(prototype diagnostic)* | hold-out grid: `leadTime > 0` and late split accepted (3/3) |
| **C′** | Curvature proxy suite internally consistent *(prototype diagnostic)* | hold-out grid: geodesic deviation tracks density or cross-proxy (3/3) |
| **I′** | Pauli stabilizer generators on branch subspace | ≥1 generator, distance ≥1, window scaling |
| **S′** | Defect more localized in ordered phase | `orderedLongerLived` on h=0.5 vs 2.5 |
| **T′** | Branch Born weights co-move with distinguishability | Env-entropy vs overlap ρ or strong imbalance ρ |
| **U′** | EFT DOF per site: two-of-three agreement | Branch effective rank, stabilizer code rate, and window participation ratio — ≥2 pairs agree within tolerance |
| **V′** | In-place tensor split relieves pressure *(prototype diagnostic)* | hold-out grid: `inPlaceImproves` at peak-pressure trigger (3/3) |
| **W′** | Holographic bound n_min finite with plateau | `nMin` Some and `boundScales` on chain sweep; `localityOk` from blind factorization on \|ψ⟩ (not native Ĥ) |
| **X** | Code subspace identified: branches inside, mixed state less so | `codeSubspaceFound` (k≥1, branch avg fidelity >0.5) and `fidelitySelectivity > 1.1` from [[n,k,d]] stabilizer generators |
| **Y** | Multi-frame boost: 4-observer velocity CV < 0.25 | Extends P from 2 to 4 spatially distinct observer clocks (uniform, corner TL, top-edge, corner BR); CV of light-cone velocities |
| **Z** | Full Poincaré composite: rotation + 4-frame boost + dispersion all pass | `allPoincareOk` — rotation CV < 0.25 (J), boost CV < 0.25 (K), linear dispersion (H) simultaneously |
| **AA** | Locality from spectrum: MI+bandwidth blind inference recovers chain factorization | ≥67% of 9 cases (TFIM ordered h=0.5 / critical h=1.0 / paramagnet h=1.5, 3 seeds each, n=6) — no Ĥ consulted |
| **AB** | Factor count dynamics: n_opt_pressure(t) increases with entanglement light cone | `nOptPressureIncreases` and `nOptPressurePeak ≥ nStart + deltaN` on TFIM defect quench (n=4..14, dt=0.1, 40 steps) |
| **AC** | Holographic structure absent in ordered phase, emerges at critical point | `emergenceNearCritical` and `orderedPhaseNonholographic` on field sweep h=0.3..1.5; profile: None/None/None→6/6/6 at h_c≈1.0 |
| **M** | Negative controls reject fake locality | `random` n=6 and scrambled-spectrum shuffled chain do not recover |

**K**, **L**, **L′**, **O**, **P**, **Q**, **AD**, **AE**, **AF**, **R′**, **F′**, **C′**, **I′**, **S′**, **T′**, **U′**, **V′**, **W′**, **X**, **Y**, **Z**, **AA**, **AB**, **AC**, and **M** target [roadmap.md](./roadmap.md) signatures **S1**, **S4**, **S5**, **S6–S8**, **S9**, and **S10**.

## Known circularity (Phase 9 — complete)

Several **′** tests are **diagnostic self-consistency** checks, not independent physics verification. Phase 9 hardening is complete — see [circularity-audit.md](./circularity-audit.md).

| ID | Issue | Resolution |
|----|-------|------------|
| ~~**W′**~~ | ~~`locality_ok` uses native chain Ĥ — tautological~~ | **Done (2026-06-22)** — blind factorization on \|ψ⟩ |
| ~~**F′**, **V′**, **R′**, **C′**~~ | ~~Same refinement pressure / MI stack end-to-end~~ | **Done (2026-06-22)** — hold-out quench grid; relabeled prototype diagnostic in docs |
| ~~**U′**~~ | ~~Measured and predicted DOF from same branch window~~ | **Done (2026-06-22)** — third estimator: participation ratio on window spectrum; two-of-three pass |
| ~~**T′**, **I′**~~ | ~~Thresholds tuned on demo quench~~ | **Done (2026-06-26)** — hold-out grids in `matter_holdout.rs`; 3/3 pass each |

**AB** and **AC** run via `npm run bench:factor` (not yet in the main 35-test battery).

## Planned extensions

See [roadmap.md](./roadmap.md) Phase 11 — scattering phase shift, 2D/3D scattering, effective mass, factorization embedding comparison, and related items.

## Implementation

- Rust: [`wasm/mad-dog-sim/src/run_falsification.rs`](../wasm/mad-dog-sim/src/run_falsification.rs) → `run_falsification_battery_json`
- Ensemble: [`wasm/mad-dog-sim/src/run_factorization_ensemble.rs`](../wasm/mad-dog-sim/src/run_factorization_ensemble.rs) → `run_factorization_ensemble_json`
- CLI: `npm run check:falsification` → [`scripts/falsification-check.ts`](../scripts/falsification-check.ts)
- UI: [`FalsificationViz`](../src/components/FalsificationViz.tsx) (async WASM worker)

Thresholds are hand-tuned on TFIM demos; treat failures as research signals, not theorem violations.

**Refinement diagnostics (F′, V′, R′, C′):** pass criteria use a hold-out `(n, field, seed)` grid defined in [`refinement_holdout.rs`](../wasm/mad-dog-sim/src/refinement_holdout.rs) — distinct from the calibration demo (`n=10, h=1.5, seed=7711`). These tests verify **diagnostic self-consistency** of the refinement stack, not independent physics. See [circularity-audit.md](./circularity-audit.md).
