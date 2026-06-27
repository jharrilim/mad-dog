# Research Roadmap

Dated backlog for testing whether systems **behave Mad-Dog-ly** — recoverable locality, relational time, holographic entanglement, excitations, branches, and (eventually) Lorentz-ish causality — without claiming to derive our universe from axioms.

**Last updated:** 2026-06-27 (Phase 11 AD)

See also: [falsification.md](./falsification.md) (automated battery), [open-questions.md](./open-questions.md) (honest limits).

---

## Mad-Dog signatures (S1–S10)

Operational checklist — each maps to falsification tests and roadmap phases.

| ID | Signature | Battery | Phase | Status |
|----|-----------|---------|-------|--------|
| **S1** | Locality entanglement-readable | A, A′, **K**, **M** | 1 | **Shipped** — model zoo ensemble + negative controls |
| **S2** | Low-dim MI geometry | B, J, **J′** | 2 | **Shipped** — chain + 2×2×3 cube; dim sweep bench |
| **S3** | Relational time (no global clock) | D, D′, G, G′, **N** | **Shipped** — grid/cube clocks; modular vs uniform; simultaneity bend |
| **S4** | Causal light cones | E, B, **Q** | **Shipped** — scattering phase + grid isotropy |
| **S5** | Holographic entanglement | C, C′, F, F′, H, **R′** | **Shipped** — RT quench series, predictive split, geodesic proxy |
| **S6** | Particles = excitations | E, **S′** | **Shipped** — ordered-phase defect localization |
| **S7** | Classical branches | I, **T′** | **Shipped** — Born-weight / distinguishability correlation |
| **S8** | IR subspace / code-like | I, **I′**, **U′**, **X** | **Shipped** — stabilizer search + two-of-three EFT DOF + QECC code subspace ID |
| **S9** | Lorentz-ish IR causality | **L**, **L′**, **O**, **P**, **Y**, **Z** | **Shipped** — scaling, dispersion, boost invariance, multi-frame boost, Poincaré composite |
| **S10** | Factor count not arbitrary | F, H, **V′**, **W′** | **Shipped** — in-place split + blind-locality holographic n_min |

**Progress metric:** signatures pass on **models we didn’t hand-tune**, at **growing n**, with **negative controls** failing loudly.

---

## Shipped (2026-06)

| Item | Notes |
|------|-------|
| Worldline tracking + two-particle scattering | [scattering.md](./scattering.md), [emergent-particles.md](./emergent-particles.md) |
| Three-or-more-clock consistency networks | [multi-clock.md](./multi-clock.md); falsification **G** |
| Adaptive holographic refinement | [refinement.md](./refinement.md); falsification **F**, **H** |
| Spectrum-only factorization blind tests | [factorization.md](./factorization.md); falsification **A** |
| Minimal decoherence / Everett branching | [decoherence.md](./decoherence.md) |
| Larger 3D lattices + dim=3 resolution | [universe-lab.md](./universe-lab.md); `bench:universe` |
| Curved space via mass concentrations | [holography.md](./holography.md); `sweep:mass` |
| QECC / EFT subspace probe + code subspace ID | [qecc-probe.md](./qecc-probe.md); falsification **I**, **X** — `run_qecc_json` |
| Gauge-free MI geometry stability | [geometry-stability.md](./geometry-stability.md); falsification **J** |
| Ensemble spectrum blind recovery | Falsification **K** — TFIM/XX/Heisenberg/sparse ≥80% |
| Negative controls (random + scrambled spectrum) | Falsification **M** |
| Factorization uniqueness report | `uniqueness` on search results + bench assertion |
| Model zoo (XX, Heisenberg, sparse local) | [factorization.md](./factorization.md); `bench:factorization` |
| Lorentz proxy v0 (directional isotropy) | Falsification **L** — cardinal speed CoV on 3×3 grid |
| 3D gauge-free geometry (J′) | 2×2×3 cube quench; spectral vs MDS embedding ρ |
| Dim vs manifold sweep | `bench:geometry` — chain/grid/cube ground states |
| Quench dim stability | `dimStable` on cube quench (no collapse to dim=1) |
| Grid/cube multi-clock networks | Falsification **G′**; `bench:time` |
| Modular + Z edge clocks vs uniform Δt | Falsification **D′** |
| Emergent simultaneity surface bend | Falsification **N**; `run_simultaneity_json` |
| Lorentz scaling sweep (3×3→4×4) | Falsification **L′**; `bench:causality` |
| Dispersion ω(k) linear at small k | Falsification **O** |
| Weak boost invariance (clock subset) | Falsification **P** |
| Scattering exchange phase stability | Falsification **Q** |

### Phase 5 — Holography under dynamics (S5) — shipped 2026-06-21

| Item | Notes |
|------|-------|
| RT time series under defect quench | `run_rt_quench_json`; falsification **R′** |
| Predictive refinement early warning | `run_predictive_refinement_json`; falsification **F′** |
| Geodesic deviation curvature proxy | `run_curvature_proxy_quench_json`; falsification **C′** |
| Holography dynamics bench | `npm run bench:holography` |

### Phase 6 — Matter and classicality (S6–S8) — shipped 2026-06-21

| Item | Notes |
|------|-------|
| Pauli stabilizer search on branch window | `run_stabilizer_search_json`; falsification **I′** |
| Defect lifetime ordered vs disordered | `run_particle_stability_json`; falsification **S′** |
| Branch Born-weight consistency | `run_branch_born_json`; falsification **T′** |
| EFT DOF two-of-three agreement | `run_eft_dimension_json`; falsification **U′** (participation-ratio third estimator) |
| Matter bench | `npm run bench:matter` |

### Phase 7 — Factor count dynamics (S10) — shipped 2026-06-21

| Item | Notes |
|------|-------|
| In-place tensor split | `run_inplace_split_json`; falsification **V′** (hold-out hardened Phase 9) |
| Holographic bound on n | `run_holographic_bound_json`; falsification **W′** (blind factorization locality) |
| Factor dynamics bench | `npm run bench:factor` |

### Phase 8 — Observational bridge — shipped 2026-06-21

| Item | Notes |
|------|-------|
| Lorentz ε(n) vs GRB/synchrotron caps | `scripts/observational-lorentz.ts`; `exports/observational/lorentz.json` |
| RT calibration vs AdS/CFT anchor | `scripts/observational-holography.ts` |
| Multi-clock desync export | `scripts/observational-time.ts` |
| Stabilizer code rate vs noise floors | `scripts/observational-ir.ts` |
| Orchestrator | `npm run bench:observational` |
| Doc | [observational-bridge.md](./observational-bridge.md) |

### Phase 9 — Circularity hardening — **complete** — 2026-06-26

| Item | Notes |
|------|-------|
| W′ blind locality on \|ψ⟩ | `holographic_bound.rs` — factorization search, not native Ĥ |
| F′/V′/R′/C′ hold-out grid | `refinement_holdout.rs`; battery pass on 3/3 hold-outs; relabeled diagnostic |
| U′ third DOF estimator | `eft_dof.rs` — participation ratio; two-of-three agreement |
| T′ Born hold-out | `matter_holdout.rs`; 3/3 hold-out (n, field, couple_step) grid; battery pass |
| I′ stabilizer hold-out | `matter_holdout.rs`; 3/3 hold-out (coupling, window_radius) grid; battery pass |
| Circularity index complete | [circularity-audit.md](./circularity-audit.md) — all ′ tests indexed |

---

### Phase 10 — Factor count dynamics — **shipped 2026-06-26**

| Item | Notes |
|------|-------|
| Locality from spectrum (blind) | `locality_spectrum.rs`; falsification **AA**; 9/9 TFIM cases recover |
| 3D space emergence (2×2×2) | `estimate_emergent_dimension` requires ≥5% scree gap; dim=3 on all cubes |
| Weight-3 stabilizers | `candidate_strings` includes ZZZ/XXX triples; `mixed_pauli_expectation` weight-3 |
| n_opt_pressure(t) time series | `factor_count_dynamics.rs` — TFIM defect quench, n=4..14, dt=0.1 |
| Falsification AB | `nOptPressurePeak ≥ nStart + deltaN`; staircase 4→8→10→14 confirmed |
| Field-sweep holographic emergence | `run_field_sweep` in `holographic_bound.rs`; falsification **AC** |
| Falsification AC | `emergenceNearCritical` + `orderedPhaseNonholographic`; h=0.3..0.7→None, h=1.0..1.5→6 |

---

### Phase 11 (partial) — Scattering extensions — **shipped 2026-06-27**

| Item | Notes |
|------|-------|
| Overlap-localized phase shift + time delay | `analyze_interaction` in `scattering.rs` |
| Falsification AD | `overlapDetected`, \|phaseShift\| > 0.05, finite `separationTimeDelay` |
| UI phase chart + overlap marker | `ScatteringViz.tsx` |
| h sweep columns | `sweep:scattering` reports phaseShift, timeDelay |
| Two-defect scattering on grid/cube | `kind: grid|cube`; Manhattan separation; falsification **AE** |
| Effective mass from dispersion (chain) | `effectiveMass` from ω² = m² + v²k²; falsification **AF** |
| UI lattice selector | Chain / 3×3 grid / 2×2×2 cube presets |

---

## Next up — Phase 11 (physics extensions)

Phase 11 scattering items **shipped 2026-06-27**. Remaining backlog:

| Priority | Item | Notes | Doc |
|----------|------|-------|-----|
| **Medium** | **Same \|ψ⟩, different `n`** | Compare factorizations via explicit embedding / truncation across factor counts | [open-questions.md](./open-questions.md) |
| **Medium** | **3D / torus factorization search** | Builders exist; permutation search is line/grid only today | [factorization.md](./factorization.md) |
| **Lower** | **QECC on non-TFIM models** | Extend code-subspace probe beyond TFIM decoherence quench | [qecc-probe.md](./qecc-probe.md), [open-questions.md](./open-questions.md) |
| **Lower** | **Modular / QFT-style clock graphs** | Z-threshold clocks on lattices shipped; modular-flow networks on grids/cubes not wired | [multi-clock.md](./multi-clock.md), [emergent-time.md](./emergent-time.md) |
| **Research** | **True eigenvalue-only blind inference** | Spectrum mode still uses eigenvector amplitudes; no {Eₙ}-only recovery | [factorization.md](./factorization.md), Phase 1 deferred |

**Suggested first pick:** same |ψ⟩ factorization comparison across factor counts.

## Deferred (not rejected)

| Item | Why deferred |
|------|--------------|
| WebGPU backend | WASM sufficient for current lattice sizes |
| Code-split `/lab` bundle | Engineering; not physics-blocking |
| 3×3×3 cube in browser | Hilbert space too large — see [non-goals](../.cursor/memory/non-goals.md) |

---

## How to use this doc

- Move rows to **Shipped** with a date when done.
- Add falsification letters when a claim becomes automated (**K**, **L** added 2026-06-21).
- Cross-link detailed notes in `docs/`; keep `sim-check.ts` / falsification battery aligned with any **OK** claims.
- **Winning** looks like: Phase 11 items shipped with falsification tests where claims sharpen, pass rates improving with n, negative controls failing.
