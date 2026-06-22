# Research Roadmap

Dated backlog of experiments and features we want to explore. Update when items ship or priorities shift.

**Last updated:** 2026-06-21

## In progress / recently shipped

| Item | Status | Notes |
|------|--------|-------|
| Worldline tracking + two-particle scattering | **Shipped (2026-06)** | See [scattering.md](./scattering.md), [emergent-particles.md](./emergent-particles.md). |
| Three-or-more-clock consistency networks | **Shipped (2026-06)** | See [multi-clock.md](./multi-clock.md). |
| Adaptive holographic refinement — dynamical splitting | **Shipped (2026-06)** | Split trigger + n vs n+Δ accept rule. See [refinement.md](./refinement.md). |
| Spectrum-only factorization — harder blind tests | **Shipped (2026-06)** | Permutation-aware spectrum scorer; n=4,6,8 recovery in `bench:factorization`. See [factorization.md](./factorization.md). |
| Minimal decoherence / Everett branching | **Shipped (2026-06)** | `run_decoherence_quench_json`, branch overlays. See [decoherence.md](./decoherence.md). |
| Larger 3D lattices + dim=3 resolution | **Shipped (2026-06)** | 2×2×3 resolves dim=3; Kaiser+gap estimator; `bench:universe`. See [universe-lab.md](./universe-lab.md). |
| Curved space via controlled mass concentrations | **Shipped (2026-06)** | Multi-insertion density sweep; `sweep:mass`; plot in RtMassViz. See [holography.md](./holography.md). |
| QECC / EFT subspace probe | **Shipped (2026-06)** | Branch-resolved Pauli sharpness + window rank; falsification **I**. See [qecc-probe.md](./qecc-probe.md). |
| Gauge-free MI geometry stability | **Shipped (2026-06)** | Distance-matrix Spearman across slices; falsification **J**. See [geometry-stability.md](./geometry-stability.md). |

## Next up (ranked)

_Phase 2 foundations shipped — add new experiments here as they emerge._

| Item | Notes |
|------|-------|
| Stabilizer search on excitation subspace | Explicit Pauli-group scan beyond sharpness heuristic |
| 2D/3D geometry stability | Extend gauge-free diagnostics to grid/cube quenches |
| Scattering phase shifts | JW-resolved exchange beyond separation-vs-time |

## Deferred (not rejected)

| Item | Why deferred |
|------|--------------|
| WebGPU backend | WASM sufficient for current lattice sizes |
| Lorentz invariance test | Large scope |
| In-place tensor factor split | Honest n+Δ comparison only for now |
| Code-split `/lab` bundle | Engineering; not physics-blocking |

## How to use this doc

- Move rows to **Shipped** with a date when done.
- Add new rows under **Next up** when experiments suggest themselves.
- Cross-link detailed notes in `docs/`; keep `sim-check.ts` / falsification battery aligned with any **OK** claims.
