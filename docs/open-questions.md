# Open Questions

## From the paper (not yet simulated)

- **Lorentz invariance** — no finite-dimensional unitary Lorentz reps on factors; how approximate is emergent Lorentz symmetry?
- **Effective field theory / QECC** — are IR matter degrees of freedom a code subspace?
- **Problem of time** — what picks the clock subsystem? (We use explicit Δt ticks or site-based physical clocks — not derived from Wheeler–DeWitt constraint.)
- **Locality from spectrum** — search for factorizations that make a given Ĥ look local.
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

**What we could test (not built):**

- Monitor RT slope and area-law saturation as entanglement grows under a quench; ask at what effective complexity a larger `n` would be *required* to restore area-law scaling in the MDS embedding.
- Compare factorizations of the same |ψ⟩ with different `n` (tensor network truncation) and measure when holography breaks.

**Prototype (2025):** see [factorization.md](./factorization.md) — permutation search over line factorizations in `src/sim/factorization.ts` / WASM `factorization.rs`.

**Prototype (2025):** see [refinement.md](./refinement.md) — quench pressure tracker + n vs n+Δ comparison in `src/sim/refinement.ts` and `RefinementViz`. Still no dynamic factor splitting.

**Ranking (honest):**

| Hypothesis | In the paper today | In our sim today |
|------------|-------------------|------------------|
| Fixed ℋ, fixed `n` | Yes | Yes (by design) |
| Factorization from spectrum | Open | **Prototype search** (line graph, n≤8 exact) |
| Holographic bound on factor count | Thematic only | Baby RT / area law only |
| Adaptive refinement | Extension | **Prototype diagnostics** (no splitting) |

## From our experiments

| Question | Status |
|----------|--------|
| Does 3D space emerge on a 3D lattice? | MDS gives 3 coords; dim detector says 2 at n=8. Inconclusive. |
| Does holographic area law hold? | Yes for ground state vs random. |
| Does discrete RT hold? | Yes, slope ≈ 1, R² ≈ 0.99. |
| Does mass deform RT slope? | Yes, slope rises to ~1.4. |
| Is time absolute? | No — two clocks desynchronize (edge vs uniform). |
| What does an emergent particle look like? | Domain-wall worldline; see [emergent-particles.md](./emergent-particles.md). |

## Engineering limits

- **Hilbert space size** — exponential in qubits; 3×3×3 cube impractical in browser TS.
- **MDS gauge freedom** — Procrustes needs `truePositions`; without them, embeddings spin between frames.
- **Ground vs quench** — one initial condition cannot optimize all tests (holography wants ground state; spacetime wants quench).
- **Decoherence** — branching/classicality not in live engine.
- **Bundle size** — Three.js adds ~1.3MB; lab route could be code-split.

## Roadmap ideas (from project)

- WebGPU backend validated against TS reference.
- Three-or-more-clock consistency networks.
- Worldline tracking and two-particle scattering.
- Curved space via stronger controlled mass concentrations.
- Code-split `/lab` for faster essay page load.

## How to extend notes

Add dated entries under `docs/` when running new experiments. Keep `sim-check.ts` assertions in sync with any claim marked **OK** in these notes.
