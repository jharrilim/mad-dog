# Open Questions

## Signature program (does the universe behave Mad-Dog-ly?)

We are **not** trying to prove Mad-Dog from axioms or solve the measurement problem. We **are** building a falsifiable checklist of structural behaviors Carroll–Singh emergence requires — see signatures **S1–S10** in [roadmap.md](./roadmap.md).

| Layer | What we can do computationally |
|-------|--------------------------------|
| **Necessary conditions** | Falsification battery A–L; extend as claims sharpen |
| **Toy demonstrations** | Universe Lab, scattering, decoherence, factorization search |
| **Scaling / uniqueness** | Open — biggest gap between “mechanism” and “our-universe-likeness” |
| **Observational export** | Map sim thresholds to Lorentz / entropy bounds | **Shipped** — [observational-bridge.md](./observational-bridge.md), `npm run bench:observational` |

Automated today: falsification battery **A–AI** (41 tests in `check:falsification`; **AB**, **AC** in `bench:factor`). Signature program **S1–S10** and Phase 10 complete.

**Next ranked (Phase 11):** Modular / QFT-style clock graphs. See [roadmap.md](./roadmap.md).

## From the paper (not yet simulated)

- **Lorentz invariance** — no finite-dimensional unitary Lorentz reps on factors; how approximate is emergent Lorentz symmetry?
- **Effective field theory / QECC** — are IR matter degrees of freedom a code subspace? **Shipped:** [qecc-probe.md](./qecc-probe.md) — branch fidelity > 0.5 with [[n,k,d]] subspace; mixed-state selectivity > 1.1; falsification test X; weight-3 stabilizers (2026-06-26). **Extended (AI):** XX and Heisenberg chains via `model` on decoherence/QECC configs.
- **Problem of time** — what picks the clock subsystem? (We use explicit Δt ticks or site-based physical clocks — not derived from Wheeler–DeWitt constraint.)
- **Locality from spectrum** — search for factorizations that make a given Ĥ look local. **Shipped (AA):** MI + bandwidth alone recovers chain factorization for TFIM ordered/critical/paramagnet (n=6, 9/9 cases); XX gapless chain fails as expected (MI is non-local in gapless phase — a correct negative). See `locality_spectrum.rs`.
- **Factor count and Hilbert-space growth** — Mad-Dog takes ℋ = ⊗ₐ ℋₐ and the number of micro-factors as primitive. It does not explain why *this* factorization, *this* dimension, or whether new fundamental degrees of freedom can appear over time (cosmological “creation” of qubits, growing Hilbert space in quantum gravity, etc.). Our simulator fixes `n` at model-build time; it never adds a tensor factor.

## Factor count: hypotheses (speculative)

Mad-Dog does not answer factor count; these are plausible directions from prior art, plus one extension native to this project’s toolkit (entanglement geometry, locality-from-spectrum, baby holography).

### Prior art (grouped)

| Family | Idea | Fit with Carroll–Singh |
|--------|------|------------------------|
| **Static ℋ** | All factors exist from the start; “creation” is excitation and entanglement redistribution, not new ⊗ factors. | Closest to the paper as written. |
| **Spectrum-first** | Only {Eₙ} and {ψₙ} are primitive; the tensor product and factor count are *derived* by finding a factorization where Ĥ looks local. | Explicit open direction (“locality from the spectrum”). |
| **Scale-dependent decomposition** | “How many qubits?” changes under coarse-graining; same global state, different effective site count. | Consistent with non-unique factorization. |
| **QFT creation** | Particles are excitations of modes already in ℋ, not new fundamental factors. | Matches emergent-particle / quench demos. |
| **Environmental branching** | Effective dimension grows for subsystems as they entangle with environments; global ℋ may stay fixed. | Related to emergent classicality, not new ℋₐ. |
| **Dynamic Hilbert space (QG)** | Cosmology, causal sets, spin foams: combinatorial structure — and Hilbert space dimension — grows over time. | Not developed in the paper; different formalism. |
| **Holographic bounds** | Degrees of freedom in a region track boundary area (covariant entropy bound), not a pre-labeled qubit lattice. | Thematic fit via area law / RT tests; factor count tied to emergent geometry. |
| **Emergent Fock space** | Second quantization and occupation numbers are IR effective; ties to QECC / EFT subspace question. | Open question in the paper. |

### Mad-Dog-native extension: adaptive holographic refinement

**Claim (speculative):** factor count is not primitive. It is the size of the *minimal* local factorization that can represent the current |ψ⟩ while satisfying holographic-style constraints.

Sketch:

1. Primitive data stay |ψ⟩ and Ĥ (or spectrum + state components).
2. Among factorizations where Ĥ is approximately local (graph-like), prefer those that minimize factor count subject to:
   - emergent MI geometry being low-dimensional (MDS / emergent-dim detector), and
   - region entropies obeying an area law and a discrete RT-type relation (see [holography.md](./holography.md)).
3. **Cosmological “new qubit”** means: no factorization with the *old* count can simultaneously keep Ĥ local, satisfy those entanglement inequalities, and represent |ψ(t)⟩ with bounded effective bond dimension. The universe **refines** the tensor network — splits a factor, adds a link — like adaptive mesh refinement.

In simulator terms: instead of hand-picking `tfimChain(n+1)`, detect failure of area-law / RT / emergent-dim diagnostics on the current state under a fixed `n`, then add a factor by a dynamical splitting rule. “Expansion” becomes growth of the minimal holographically consistent factor graph.

**What we tested (shipped 2026-06-26 — falsification AB):**

- n_opt_pressure(t): the chain size minimising holographic pressure tracks the entanglement light cone — staircase 4→6→8→10→14 during a TFIM defect quench. See `factor_count_dynamics.rs`, `bench:factor`.
- Honest finding: binary n_min (RT/area-law pass/fail) is too strict for fast quench dynamics (all chains fail simultaneously when volume-law entanglement arrives). The pressure-based signal is more robust.

**What we could still test (not built):**

- ~~Compare factorizations of the same |ψ⟩ with different `n` via explicit embedding / truncation.~~ **Shipped (AG)** — `factorization_compare.rs`, `FactorizationCompareViz.tsx`; roundtrip F > 0.99, locality/dim stable under embed.
- Field-sweep n_min: **Shipped (AC)** — holographic RT structure absent for h<1 (ordered/cat-state phase), emerges at h≈1.0. Profile: None/None/None→6/6/6. The transition in holographic structure coincides with the TFIM phase transition. n_min does not vary above h_c (all n=6 in paramagnet), so the "more factors near criticality" hypothesis is not supported — instead the story is a phase transition in holographic structure itself.

**Prototype:** see [factorization.md](./factorization.md) — permutation search (line/grid), multi-eigenstate + spectrum modes, annealing for n>8, joint quench study with [refinement.md](./refinement.md).

**Ranking (honest):**

| Hypothesis | In the paper today | In our sim today |
|------------|-------------------|------------------|
| Fixed ℋ, fixed `n` | Yes | Yes (by design) |
| Factorization from spectrum | Open | **Search prototype** (Pauli/spectrum, line/grid, annealing) |
| Holographic bound on factor count | Thematic only | Baby RT / area law only |
| Adaptive refinement | Extension | **Split heuristic** + n vs n+Δ accept rule |
| Factor count dynamics (n_opt_pressure) | Extension | **Shipped (AB)** — pressure-optimal chain tracks entanglement light cone |

## From our experiments

| Question | Status |
|----------|--------|
| Does 3D space emerge on a 3D lattice? | **Yes** — dim=3 on both 2×2×2 and 2×2×3 ground states (h=1.5). Root cause of prior failure: T₁u eigenvalue degeneracy in MI Gram matrix caused floating-point scree-gap at 1+2e-14, fixed by requiring ≥5% ratio. See [universe-lab.md](./universe-lab.md). |
| Does holographic area law hold? | Yes for ground state vs random. |
| Does discrete RT hold? | Yes, slope ≈ 1, R² ≈ 0.99. |
| Does mass deform RT slope? | Yes; Δslope grows with excitation density ρ (RT fit degrades at high ρ). See [holography.md](./holography.md), `sweep:mass`. |
| Is time absolute? | No — two clocks desynchronize (edge vs uniform). |
| What does an emergent particle look like? | Domain-wall worldline with centroid overlay; two-defect scattering on Experiments page. See [emergent-particles.md](./emergent-particles.md), [scattering.md](./scattering.md). |

## Engineering limits

- **Hilbert space size** — exponential in qubits; 3×3×3 cube impractical in browser TS.
- **MDS gauge freedom** — Procrustes needs `truePositions`; without them, embeddings spin between frames.
- **Ground vs quench** — one initial condition cannot optimize all tests (holography wants ground state; spacetime wants quench).
- **Decoherence** — minimal env-coupling quench with branch-resolved worldlines ([decoherence.md](./decoherence.md)); QECC subspace ID still open.
- **Bundle size** — Three.js adds ~1.3MB; lab route could be code-split.

## Roadmap

Phased attack order and signature map: [roadmap.md](./roadmap.md).

## How to extend notes

Add dated entries under `docs/` when running new experiments. Keep `sim-check.ts` assertions in sync with any claim marked **OK** in these notes.
