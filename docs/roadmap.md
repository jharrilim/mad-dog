# Research Roadmap

Dated backlog for testing whether systems **behave Mad-Dog-ly** — recoverable locality, relational time, holographic entanglement, excitations, branches, and (eventually) Lorentz-ish causality — without claiming to derive our universe from axioms.

**Last updated:** 2026-06-21

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
| **S8** | IR subspace / code-like | I, **I′**, **U′** | **Shipped** — stabilizer search + EFT DOF counting |
| **S9** | Lorentz-ish IR causality | **L**, **L′**, **O**, **P** | **Shipped** — scaling, dispersion, boost invariance |
| **S10** | Factor count not arbitrary | F, H | 7 | Split heuristic; in-place split deferred |

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
| QECC / EFT subspace probe | [qecc-probe.md](./qecc-probe.md); falsification **I** |
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
| EFT DOF vs stabilizer code rate | `run_eft_dimension_json`; falsification **U′** |
| Matter bench | `npm run bench:matter` |

---

## Next up (ranked execution order)

### Phase 7 — Factor count dynamics (S10)

| Priority | Item | Deliverable | Pass criterion |
|----------|------|-------------|----------------|
| 1 | In-place tensor split | Same \|ψ⟩, split one factor without n+Δ re-run | Diagnostics improve at same bond budget |
| 2 | Holographic bound on n | Min n s.t. area law + dim + locality hold | Scaling law n_min vs system size |

### Phase 8 — Observational bridge (export, not WASM)

| Priority | Item | Role |
|----------|------|------|
| 1 | Lorentz violation bounds | Map Phase 4 ε(n) to GRB / synchrotron caps |
| 2 | Holographic entropy | Calibrate RT failures vs AdS/CFT analogues |
| 3 | Problem of time | Export multi-clock desync as frozen-formalism metaphor |
| 4 | IR subspace | Connect stabilizer rates to lab noise thresholds |

---

## Deferred (not rejected)

| Item | Why deferred |
|------|--------------|
| WebGPU backend | WASM sufficient for current lattice sizes |
| Full Lorentz / Poincaré test | Beyond **L** v0; needs dispersion + scaling |
| In-place tensor factor split | Honest n+Δ comparison only for now |
| Code-split `/lab` bundle | Engineering; not physics-blocking |

---

## How to use this doc

- Move rows to **Shipped** with a date when done.
- Add falsification letters when a claim becomes automated (**K**, **L** added 2026-06-21).
- Cross-link detailed notes in `docs/`; keep `sim-check.ts` / falsification battery aligned with any **OK** claims.
- **Winning** looks like: more signatures on non-TFIM models, pass rates improving with n, negative controls failing, observational thresholds documented.
