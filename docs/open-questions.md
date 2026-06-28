# Open Questions

Honest limits and unresolved Carroll–Singh themes. **Active backlog:** [roadmap.md](./roadmap.md) Phase 12.

We are **not** proving Mad-Dog from axioms or solving the measurement problem. We **are** testing whether toy systems exhibit the structural behaviors the emergence program needs — signatures **S1–S10** in the roadmap.

---

## Where we stand

| Layer | Status |
|-------|--------|
| **Necessary conditions (S1–S10)** | Shipped — falsification battery **A–AN** (46 tests in `check:falsification`; **AB**, **AC** in `bench:factor`; `check:scaling` for n-sweep + G boundary) |
| **Toy demonstrations** | Universe Lab, scattering, decoherence, factorization, clocks, holography |
| **Scaling / uniqueness** | **Partially closed** — `check:scaling` (16 checks): factorization + AA-blind chain n=4–8, AA-blind 2D grid/torus, XX gapless negative, uniqueness zoo; multi-clock G breaks n≥11 (**AM**) |
| **Observational export** | Shipped — [observational-bridge.md](./observational-bridge.md), `npm run bench:observational` |

**Progress metric (from roadmap):** signatures pass on models we didn’t hand-tune, at **growing n**, with **negative controls** failing loudly.

---

## Answered in the simulator (pointers)

| Question | Answer | Where |
|----------|--------|-------|
| Does 3D space emerge on a 3D lattice? | Yes (dim=3 on 2×2×2 and 2×2×3) | [universe-lab.md](./universe-lab.md) |
| Holographic area law / discrete RT? | Yes on ground; slope ≈ 1 | [holography.md](./holography.md) |
| Mass deforms RT slope? | Yes vs excitation density ρ | `sweep:mass` |
| Is time absolute? | No — relational / multi-clock desync | [emergent-time.md](./emergent-time.md), **D**, **G**, **AJ** |
| Emergent particles? | Domain-wall worldlines; two-defect scattering | [emergent-particles.md](./emergent-particles.md), [scattering.md](./scattering.md) |
| Blind locality from spectrum (chain)? | Yes for TFIM phases; XX gapless fails (correct negative) | **AA**, `locality_spectrum.rs` |
| Blind locality from {Eₙ} only? | **No** — eigenvalues permutation-invariant | **AK**, [factorization.md](./factorization.md) |
| QECC / code-like IR subspace? | Proto yes — **X**, **AI** on tuned chains | [qecc-probe.md](./qecc-probe.md) |
| Lorentz-ish causality? | Proxies only — **S9** battery | [falsification.md](./falsification.md) |

---

## Still open (conceptual)

From Carroll–Singh and our experiments — not closed by current code.

### Scaling and uniqueness

- ~~Are factorization search results **unique** enough (`equivalenceClassCount`, class score gaps) to claim labeling isn’t arbitrary?~~ **Answered (2026-06-27)** — `check:scaling` uniqueness checks on TFIM n=4/6/8 + XX/sparse n=6; gap narrows with n, stays positive; reflection-only degeneracy.
- Do falsification tests keep passing as **n** and lattice size grow, or do we rely on annealing luck? **Partially** — exact search to n=8; G breaks n≥11 (**AM**).

### Spectrum-first locality

- **ψₙ amplitudes required** — **AK** proved {Eₙ} alone cannot discriminate qubit labelings.
- ~~**2D blind recovery** — AA is chain-only; grid/torus spectrum search not in battery.~~ **Answered (2026-06-27)** — **AL** passes: 3×3 grid + 2×2 torus AA-blind (D₄ recovery metric + small-torus MI penalty). See [factorization.md](./factorization.md).
- ~~**Gapless models** — XX fails MI+bandwidth blind search; fundamental limit or fixable with richer blind data?~~ **Answered (2026-06-27)** — **AN** documents fundamental limit: gapless XX (h/J < 1) has algebraically-decaying MI; top-20 blind scores tie (spread < 0.02). Semi-blind (with Ĥ term) still recovers XX; fix requires data beyond MI+bandwidth on ψₙ.
- **Minimal extra data** — what is the weakest observable beyond {Eₙ} that recovers labeling?

### Factor count and Hilbert-space growth

Mad-Dog takes ℋ = ⊗ₐ ℋₐ and factor count as primitive. Open:

- Why *this* factorization and dimension?
- Can fundamental degrees of freedom **appear over time** (cosmological qubit creation, growing Hilbert space)?
- Our sim **fixes `n` at build time** — never adds a tensor factor dynamically.

**Hypothesis families** (see roadmap Phase 12):

| Family | Idea |
|--------|------|
| Static ℋ | All factors exist; “creation” is excitation redistribution |
| Spectrum-first | Tensor product derived from locality of Ĥ given {Eₙ, ψₙ} |
| Scale-dependent decomposition | Effective site count changes under coarse-graining |
| Holographic bounds | DOF track boundary area, not pre-labeled qubits |
| Adaptive refinement | Factor count = minimal local decomposition satisfying RT/area-law/dim |
| Dynamic Hilbert space (QG) | Combinatorial growth — different formalism, not in sim |

**What we already tested on factor count:**

- **AB** — `n_opt_pressure(t)` tracks entanglement light cone (staircase 4→14).
- **AC** — holographic RT structure emerges at h≈1.0 phase transition; not “more factors near criticality.”
- **AG** — same \|ψ⟩ stable under embed/truncate across `n`.
- **Not built** — automatic split when RT/area-law/dim diagnostics fail on fixed `n` (roadmap Phase 12).

### Time

- **What picks the clock subsystem?** We inject uniform Δt or site/modular clocks by hand — not derived from a Wheeler–DeWitt constraint.
- Z-threshold vs modular-flow networks: when do they disagree predictably (beyond hand-tuned R² thresholds)?

### Matter / IR / branches

- **Emergent Fock space** — are occupation numbers IR-effective vs branch counting?
- **QECC at scale** — code subspace ID on larger n, 2D windows, longer quenches?
- **Born vs distinguishability (T′)** — hold-out passed; does correlation survive wider grids?

### Lorentz (paper vs sim)

- No finite-dimensional unitary Lorentz reps on factors in the paper.
- We have **S9 proxies** (cardinal CoV, dispersion, boost, Poincaré composite) — not a full emergent Lorentz derivation.

---

## Engineering limits

- **Hilbert space** — exponential in qubits; 3×3×3 cube impractical in browser ([non-goals](../.cursor/memory/non-goals.md)).
- **MDS gauge freedom** — Procrustes alignment uses known lattice layout for viz stability.
- **Ground vs quench** — one initial condition cannot optimize every test.
- **Bundle size** — Three.js ~1.3MB; `/lab` could be code-split (deferred).

---

## How to extend

1. Add Phase 12 rows to [roadmap.md](./roadmap.md) when work ships; add falsification letters when claims automate.
2. Add dated experiment notes under `docs/` when behavior changes materially.
3. Keep `npm run check:wasm` / `check:falsification` aligned with any **OK** claims.
