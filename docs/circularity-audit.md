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

## Other tests (partial index)

| ID | Independence | Notes |
|----|--------------|-------|
| **W′** | Mad-Dog | **Fixed 2026-06-22** — blind factorization locality on \|ψ⟩; hold-out ground seeds |
| **U′** | Mad-Dog | **Fixed 2026-06-22** — participation-ratio third estimator; two-of-three agreement |
| **A**, **K**, **M** | Mad-Dog | Blind / ensemble factorization — Phase 1 guards |
| **F**, **H** | Diagnostic | Same refinement stack as F′ family; not yet hold-out hardened |
| **T′**, **I′** | Circular (planned) | Thresholds tuned on demo quench |
