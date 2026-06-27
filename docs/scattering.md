# Two-defect scattering and worldline tracking

## What we built (2026-06)

Proto-particle diagnostics on TFIM quenches:

1. **Single-defect worldline** — peak signal in a half-chain window (1D) or global peak (3D cube). Enabled on chain spacetime, 2D spacetime, and Universe Lab cube runs.
2. **Dual-defect scattering** — two flipped spins, half-chain peak tracking (left/right) to avoid cross-talk. Metrics: separation series, fitted site velocities, `bothMoved`, `crossed`, `minSeparation`, exchange-phase series, overlap-localized **phase shift** and **time delay** (Phase 11, 2026-06-27).

WASM: `track_single_worldline` in `spacetime.rs`, `run_two_defect_scattering` in `scattering.rs`.

UI:

- **SpacetimeViz** — worldline overlay on heatmap
- **ScatteringViz** — dual overlays + separation and phase charts (overlap marker), h slider
- **Universe Lab** — heatmap overlay + 3D emergent path through MDS coords

CLI: `npm run sweep:scattering` (lite mode, parallel workers).

Falsification tests **E**, **Q**, **AD** (cone-overlap phase shift).

## Interaction metrics (Phase 11)

When two light cones overlap, we localize diagnostics at the separation minimum:

| Metric | Definition |
|--------|------------|
| `overlapStep` | Slice index `k*` where `separationSeries[k*]` is minimum (after 15% burn-in) |
| `overlapDetected` | `minSeparation < separationSeries[0] - 1` — cones closed vs initial spacing |
| `interactionPhaseShift` | Unwrapped exchange phase at `k*` minus linear extrapolation from pre-overlap window (`k < k* - 3`) |
| `separationTimeDelay` | `(k* - k_pred) × dt` where `k_pred` solves a linear fit of early separation vs time |

**Exchange phase** on the two-defect subspace:

δ = arg(ψ₁₁) + arg(ψ₀₀) − arg(ψ₁₀) − arg(ψ₀₁)

**Default demo** (`n=12`, `h=0.7`, defects `[3,8]`, `dt=0.12`, 40 steps): `overlapStep=22`, `interactionPhaseShift≈2.85` rad, `separationTimeDelay≈-3.72` (overlap arrives earlier than free-propagation extrapolation), `minSeparation=1`.

Falsification **AD**: `overlapDetected && |interactionPhaseShift| > 0.05 && finite separationTimeDelay`.

`npm run sweep:scattering` reports `phaseShift` and `timeDelay` across h values.

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

## Open follow-ups (Phase 11)

Tracked on [roadmap.md](./roadmap.md) Phase 11:

1. ~~**Scattering phase shift / time delay**~~ — **Shipped 2026-06-27** — `analyze_interaction` in `scattering.rs`; falsification **AD**; UI phase chart + h sweep columns.
2. **Two defects on 2D grid or 3D cube** — scattering runner is chain-only today.
3. **Effective mass** from dispersion relation of lattice excitations (extends falsification **O**).
