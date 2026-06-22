# Two-defect scattering and worldline tracking

## What we built (2026-06)

Proto-particle diagnostics on TFIM quenches:

1. **Single-defect worldline** — peak signal in a half-chain window (1D) or global peak (3D cube). Enabled on chain spacetime, 2D spacetime, and Universe Lab cube runs.
2. **Dual-defect scattering** — two flipped spins, half-chain peak tracking (left/right) to avoid cross-talk. Metrics: separation series, fitted site velocities, `bothMoved`, `crossed`, `minSeparation`.

WASM: `track_single_worldline` in `spacetime.rs`, `run_two_defect_scattering` in `scattering.rs`.

UI:

- **SpacetimeViz** — worldline overlay on heatmap
- **ScatteringViz** — dual overlays + separation chart, h slider
- **Universe Lab** — heatmap overlay + 3D emergent path through MDS coords

CLI: `npm run sweep:scattering` (lite mode, parallel workers).

Falsification test **E**: both excitations propagate, min separation ≥ 1, no binding in default TFIM demo.

## Tracking methods

| Case | Method |
|------|--------|
| Single defect (1D) | Peak signal in outbound half-chain (right front if defect at center) |
| Single defect (3D) | Global peak signal site |
| Two defects | Max signal in left half / right half independently |

Half-chain split avoids assigning both bumps to one tracker. Not JW fermion exchange — `crossed` means centroid labels swapped sides, not a resolved pass-through.

## Parameter guide

| h | Worldlines | Scattering |
|---|------------|------------|
| 0.5–0.8 (ordered) | Sharp bands | Clear separation series |
| ≈ 1.0 | Compromise | Default Universe Lab |
| 1.2+ (paramagnetic) | Muddy | Tracks smear; minSep may stay large but signal weak |

See [parameters-and-phases.md](./parameters-and-phases.md).

## Open follow-ups

- Phase shift / time delay when cones overlap (fit separation minimum vs h).
- Two defects on 2D grid or 3D cube (currently chain-only scattering runner).
- Effective mass from dispersion relation of lattice excitations.
- See [roadmap.md](./roadmap.md) for full backlog.
