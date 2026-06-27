# Mad-Dog Simulator — Research Notes

Informal notes from building and testing the interactive simulator for [Carroll & Singh (2018)](https://arxiv.org/abs/1801.08132), *Mad-Dog Everettianism: Quantum Mechanics at Its Most Minimal*.

These are **working learnings**, not claims of new physics. They record what the code does, what we measured, and what seemed to work or fail.

## Index

| Note | Topic |
|------|--------|
| [ontology-and-pipeline.md](./ontology-and-pipeline.md) | Minimal ontology → simulation pipeline |
| [emergent-space.md](./emergent-space.md) | Mutual information, MDS, dimension detection |
| [emergent-time.md](./emergent-time.md) | Page–Wootters time, light cones, two clocks |
| [holography.md](./holography.md) | Area law, baby Ryu–Takayanagi, mass deformation |
| [refinement.md](./refinement.md) | Adaptive factor-count diagnostics (prototype) |
| [factorization.md](./factorization.md) | Spectrum-driven locality search (prototype) |
| [parameters-and-phases.md](./parameters-and-phases.md) | What `h` and `J` mean; ordered vs paramagnetic |
| [universe-lab.md](./universe-lab.md) | 3+1 lab: cube lattice, dim=3 benchmarks |
| [emergent-particles.md](./emergent-particles.md) | What particle-like structure would look like |
| [scattering.md](./scattering.md) | Worldline tracking, two-defect scattering |
| [decoherence.md](./decoherence.md) | Minimal env coupling, branch-resolved tracks |
| [qecc-probe.md](./qecc-probe.md) | Branch-resolved excitation subspace (QECC / EFT proto) |
| [geometry-stability.md](./geometry-stability.md) | Gauge-free MI distance stability across slices |
| [multi-clock.md](./multi-clock.md) | N-clock consistency networks |
| [observational-bridge.md](./observational-bridge.md) | Phase 8: sim metrics vs literature caps (export) |
| [circularity-audit.md](./circularity-audit.md) | Phase 9: independence level per falsification test |
| [roadmap.md](./roadmap.md) | Signature program S1–S10 + phased backlog |
| [scaling-research.md](./scaling-research.md) | Phase 12 scaling probe results (2026-06-27) |
| [falsification.md](./falsification.md) | Automated Mad-Dog claim tests (A–W′) |
| [open-questions.md](./open-questions.md) | Honest limits and next experiments |

## Code map

```
src/sim/
  quantum.ts      — state vectors, Pauli Hamiltonians, ground-state finder
  geometry.ts     — RDM entropy, MI matrix, MDS, emergent dimension
  spacetime.ts    — unitary evolution, Page–Wootters slices, light cone
  holography.ts   — area law, RT fit, mass injection
  refinement.ts — adaptive factor-count pressure (prototype)
  factorization.ts — spectrum-driven locality search (types + TS fallback)
  relational-time.ts — dual-clock comparison (TS fallback)
  models.ts       — TFIM chain / grid / cube / random
  runner.ts       — UI-facing drivers

scripts/sim-check.ts — numerical regression checks
```

Run checks: `node scripts/sim-check.ts`
