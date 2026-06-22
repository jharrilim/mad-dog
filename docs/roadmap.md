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
| **S2** | Low-dim MI geometry | B, J, J′ (planned) | 2 | J on 1D chain; 3D extension next |
| **S3** | Relational time (no global clock) | D, G | 3 | Shipped |
| **S4** | Causal light cones | E, B | 4 | Shipped (1D); grid via **L** |
| **S5** | Holographic entanglement | C, F, H | 5 | Ground RT OK; quench dynamics partial |
| **S6** | Particles = excitations | E | 6 | Shipped (domain walls + scatter) |
| **S7** | Classical branches | I | 6 | Proto (decoherence quench) |
| **S8** | IR subspace / code-like | I, I′ (planned) | 6 | Sharpness heuristic; stabilizer search next |
| **S9** | Lorentz-ish IR causality | **L** (v0) | 4 | **L stub shipped** — directional CoV on 3×3 |
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

---

## Next up (ranked execution order)

### Phase 2 — Space isn’t an artifact (S2)

| Priority | Item | Deliverable | Pass criterion |
|----------|------|-------------|----------------|
| 1 | **J′** — 3D gauge-free geometry | Extend `geometry_stability.rs` to 2×2×3 quenches | mean Spearman ρ > 0.85 without Procrustes |
| 2 | Dim vs manifold sweep | 1D / 2D / 3D lattices at growing n | Estimator tracks lattice dim |
| 3 | Dim under quench + decoherence | Same stability test on product-state quenches | Predictable drift, not collapse to dim=1 |
| 4 | Cross-embedding consistency | MDS vs spectral-on-distance-matrix | Spearman between pipelines > threshold |

### Phase 3 — Deepen relational time (S3)

| Priority | Item | Deliverable | Pass criterion |
|----------|------|-------------|----------------|
| 1 | Clock networks on 2D/3D | N clocks on grid/cube | No global sync; defect still local |
| 2 | Modular vs physical clocks | Compare Page–Wootters vs site thresholds | Both desync from uniform Δt |
| 3 | Emergent simultaneity surfaces | Fit equal-time hypersurfaces in emergent coords | Surfaces bend with observer |

### Phase 4 — Causality → Lorentz (S4 → S9)

| Priority | Item | Deliverable | Pass criterion |
|----------|------|-------------|----------------|
| 1 | **L → scaling** | CoV of cardinal speeds vs n on grid/cube | CoV → 0 as n grows |
| 2 | Dispersion relation | Fourier mode propagation on emergent lattice | ω(k) linear at small k |
| 3 | Weak boost invariance | Light cones from different clock subsets | Shape invariant up to ε(n) |
| 4 | Scattering phase shifts | JW-resolved exchange beyond separation-vs-time | Stable phase after interaction |

### Phase 5 — Holography under dynamics (S5)

| Priority | Item | Deliverable | Pass criterion |
|----------|------|-------------|----------------|
| 1 | RT time series under quench | S_A / boundary MI during propagation | Structured deviation vs ρ |
| 2 | Predictive refinement | Split triggers before RT fails badly | Early warning → recovery |
| 3 | Curvature proxy honesty | Second diagnostic (e.g. geodesic deviation in MI space) | Internal consistency of proxy suite |

### Phase 6 — Matter and classicality (S6–S8)

| Priority | Item | Deliverable | Pass criterion |
|----------|------|-------------|----------------|
| 1 | **I′** — stabilizer search | Pauli-group scan on branch subspace | Generating set found; distance scales |
| 2 | Particle stability | Defect lifetime vs h, dimension | Long-lived quasiparticles in ordered phase |
| 3 | Branch Born weights | Branch overlap vs \|c_branch\|² | Correlation above threshold |
| 4 | EFT dimension counting | Effective DOF per site after tracing env | Matches code-rate prediction |

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
