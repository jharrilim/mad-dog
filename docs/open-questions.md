# Open Questions

## From the paper (not yet simulated)

- **Lorentz invariance** — no finite-dimensional unitary Lorentz reps on factors; how approximate is emergent Lorentz symmetry?
- **Effective field theory / QECC** — are IR matter degrees of freedom a code subspace?
- **Problem of time** — what picks the clock subsystem? (We use explicit Δt ticks or site-based physical clocks — not derived from Wheeler–DeWitt constraint.)
- **Locality from spectrum** — search for factorizations that make a given Ĥ look local.
- **Factor count and Hilbert-space growth** — Mad-Dog takes ℋ = ⊗ₐ ℋₐ and the number of micro-factors as primitive. It does not explain why *this* factorization, *this* dimension, or whether new fundamental degrees of freedom can appear over time (cosmological “creation” of qubits, growing Hilbert space in quantum gravity, etc.). Our simulator fixes `n` at model-build time; it never adds a tensor factor.

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
