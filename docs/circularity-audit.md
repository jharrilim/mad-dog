# Circularity audit

Short index of **independence level** per falsification test — generic QI fact, Mad-Dog-specific claim, or circular / self-consistency check.

**Last updated:** 2026-06-22

See [falsification.md](./falsification.md) for pass criteria and [roadmap.md](./roadmap.md) Phase 9 for the hardening backlog.

---

## Independence levels

| Level | Meaning |
|-------|---------|
| **Generic** | Expected for broad classes of gapped / local QI systems |
| **Mad-Dog** | Targets Mad-Dog emergence signatures with independent observables |
| **Diagnostic** | Internal consistency of a prototype diagnostic stack (not independent physics) |
| **Circular** | Pass criterion tautological given model choice (should be fixed or relabeled) |

---

## Refinement diagnostics (F′, V′, R′, C′) — Phase 9 Priority 2

All four tests share `measure_refinement_diagnostics` + `RefinementThresholds` end-to-end. They are **Diagnostic**, not independent physics verification.

| ID | Independence | Calibration (threshold tuning) | Hold-out evaluation |
|----|--------------|-------------------------------|---------------------|
| **F′** | Diagnostic | `n=10, h=1.5, seed=7711, steps=18` | 3-case grid in `refinement_holdout.rs` |
| **V′** | Diagnostic | same + `delta_n=2` | same grid |
| **R′** | Diagnostic | same, `steps=16` | same grid |
| **C′** | Diagnostic | same + `xi=1.0` | same grid |

**Tuned thresholds** (`refinement.rs`): `pressure=0.42`, `min_rt_r2=0.82`, `max_rt_slope_dev=0.35`.

**Hold-out grid** (not used for tuning):

| n | h | seed |
|---|---|------|
| 10 | 1.2 | 9001 |
| 8 | 1.8 | 3333 |
| 10 | 1.0 | 5555 |

Battery **pass/fail** is on hold-out 3/3 only; calibration run is reported in test detail for regression visibility.

**Follow-ups (Phase 9 items 4–5):**

- **T′**, **I′** — hold-out coupling / window bands
- Complete rows for all **′** tests in this doc

---

## U′ — independent EFT DOF (Phase 9 Priority 3)

**U′** compares three DOF-per-site estimates on the branch excitation window:

| Estimator | Source | Independence |
|-----------|--------|--------------|
| **Measured** | Branch effective rank (exp entropy of region spectrum) / window sites | Entanglement spectrum |
| **Predicted** | Stabilizer Pauli search `code_rate` | Pauli stabilizer stack |
| **Independent** | Branch participation ratio (1/Σλ²) / window sites | Entanglement spectrum (distinct functional from effective rank) |

**Pass:** at least **two of three** pairs agree within `rel_tol=0.5`, `abs_tol=0.35` (tightened vs pre–Phase 9 pairwise-only check).

Implementation: [`eft_dof.rs`](../wasm/mad-dog-sim/src/eft_dof.rs), wired in [`run_matter.rs`](../wasm/mad-dog-sim/src/run_matter.rs).

**Independence level:** **Mad-Dog** — third estimate does not use stabilizer `code_rate`; two-of-three guards against single-estimator circularity.

---

## T′ — Born hold-out (Phase 9 Priority 4)

**T′** checks that env branch weights co-move with branch distinguishability (Born rule
consistency) across a sweep of env-coupling strengths.

| Parameter | Calibration demo | Hold-out grid |
|-----------|-----------------|---------------|
| `n` | 8 | 10, 8, 10 |
| `field` | 1.2 | 1.0, 1.4, 1.6 |
| `couple_step` | 7 | 8, 6, 7 |

**Hold-out grid** (not used for tuning; must all differ from calibration on ≥1 axis):

| n | field | couple_step |
|---|-------|-------------|
| 10 | 1.0 | 8 |
| 8 | 1.4 | 6 |
| 10 | 1.6 | 7 |

Battery **pass/fail** is hold-out 3/3 only; calibration run reported for regression visibility.

**Independence level:** **Mad-Dog** — `born_consistent` thresholds (`entropy_overlap_corr < -0.35 || imbalance_overlap_corr.abs() > 0.55`) are evaluated on quench parameters not used during threshold tuning. **Fixed 2026-06-26.**

---

## I′ — Stabilizer hold-out (Phase 9 Priority 4)

**I′** checks that Pauli stabilizer generators are found on branch-resolved excitation windows,
with code distance scaling with window size.

| Parameter | Calibration demo | Hold-out grid |
|-----------|-----------------|---------------|
| `coupling` | 0.9 | 0.7, 0.9, 0.85 |
| `window_radius` | 2 | 2, 3, 2 |

Outer quench params are fixed at calibration values (`n=8, field=1.2, dt=0.2, steps=22, couple_step=7, seed=4242`).

**Hold-out grid** (varying `coupling` and `window_radius`):

| coupling | window_radius |
|----------|---------------|
| 0.7 | 2 |
| 0.9 | 3 |
| 0.85 | 2 |

Battery **pass/fail** is hold-out 3/3 only; calibration run reported for regression visibility.

**Independence level:** **Mad-Dog** — pass bands in `branch_stabilizer_candidate` (sharpening threshold, agreement tolerance, strength floor) are evaluated on `(coupling, window_radius)` pairs not used during calibration. **Fixed 2026-06-26.**

---

## Other tests (complete index)

| ID | Independence | Notes |
|----|--------------|-------|
| **W′** | Mad-Dog | **Fixed 2026-06-22** — blind factorization locality on \|ψ⟩; hold-out ground seeds |
| **U′** | Mad-Dog | **Fixed 2026-06-22** — participation-ratio third estimator; two-of-three agreement |
| **T′** | Mad-Dog | **Fixed 2026-06-26** — hold-out (n, field, couple_step) grid; 3/3 pass |
| **I′** | Mad-Dog | **Fixed 2026-06-26** — hold-out (coupling, window_radius) grid; 3/3 pass |
| **A**, **K**, **M** | Mad-Dog | Blind / ensemble factorization — Phase 1 guards |
| **A′** | Mad-Dog | Cross-graph structure — line vs grid / torus locality |
| **B** | Generic | MI emergent distance vs lattice LR velocity — holds broadly for local QI systems |
| **C**, **C′** | Diagnostic | RT ratio self-consistency; C′ on hold-out (n, field, seed) grid |
| **D**, **D′** | Mad-Dog | Modular clocks desync from uniform Δt; relational-time claim |
| **E** | Generic | Excitation propagation — expected for local Hamiltonians |
| **F**, **F′** | Diagnostic | Refinement decoupling; F′ on hold-out grid; both share refinement stack |
| **G**, **G′** | Mad-Dog | Multi-clock network lacks global consistency — relational-time signature |
| **H** | Diagnostic | Adaptive split accepted — same refinement stack as F′ |
| **I** | Mad-Dog | Branch excitations code-like after decoherence — partially generic |
| **J**, **J′** | Generic | Gauge-free MI geometry stable — expected for gapped local systems |
| **L**, **L′** | Mad-Dog | Directional causal isotropy and scaling — weak Lorentz proxy |
| **N** | Mad-Dog | Simultaneity surfaces bend between observers — relational-time signature |
| **O** | Generic | Dispersion ω(k) linear at small k — expected for free-boson-like excitations |
| **P** | Mad-Dog | Weak boost invariance under clock-subset observer |
| **Q** | Generic | Scattering exchange phase stable — expected for local elastic scattering |
| **R′** | Diagnostic | RT slope deficit structured vs density; hold-out (n, field, seed) grid |
| **S′** | Mad-Dog | Defect localized longer in ordered phase — particle signature |
| **V′** | Diagnostic | In-place tensor split relieves pressure; hold-out (n, field, seed) grid |
