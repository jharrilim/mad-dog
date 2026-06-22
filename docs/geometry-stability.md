# Gauge-free MI geometry stability

**Status:** Phase 2 (2026-06). Measures whether **mutual-information distance matrices** stay structurally consistent across clock slices — without Procrustes alignment or `truePositions`.

## Motivation

Emergent MDS coordinates require gauge choices (rotation, reflection). The MI-derived **distance matrix** D_ij is label-invariant: it depends only on qubit indices and the state. If “space” is real, relative distance **rankings** should persist as a defect propagates (even when absolute distances drift).

## Setup

Central defect quench on TFIM **chain**, **grid**, or **cube** (ordered phase h ≈ 1.2). At each slice k:

1. Build MI matrix → distance matrix D(k) via `mi_to_distance`.
2. Compare D(k) to D(k−1):
   - **Relative Frobenius drift** ‖D(k) − D(k−1)‖ / ‖D(k−1)‖
   - **Spearman ρ** on off-diagonal entries (gauge-free)
3. **Cross-embedding ρ** — Spearman between pairwise distances from **classical MDS** vs **Laplacian spectral** embedding on D(k).
4. Record emergent dimension and top Gram eigenvalues.

## Pass criteria

| Flag | Criterion |
|------|-----------|
| `geometryStable` | mean Spearman ρ > 0.85 across slices |
| `dimStable` | mean emergent dim ≥ expected − 1 and no slice dim = 0 |

Falsification **J** (chain), **J′** (2×2×3 cube quench).

## API

```typescript
import { runGeometryStabilityAsync } from '@/sim/runner-async'

// 3D cube quench
const result = await runGeometryStabilityAsync({
  kind: 'cube',
  rows: 2,
  cols: 2,
  lz: 3,
  field: 1.2,
  dt: 0.2,
  steps: 20,
})
```

Dim vs manifold sweep: `run_geometry_dim_sweep_json` — see `npm run bench:geometry`.

## Measured (default configs)

| Lattice | mean ρ | embed ρ | dim mean |
|---------|--------|---------|----------|
| chain n=10 | ≈ 0.93 | ≈ 0.95 | ≈ 1 |
| cube 2×2×3 | > 0.85 | > 0.80 | ≈ 3 |

## UI / falsification

- **Experiments** → *Gauge-free MI geometry stability* — chain / grid / cube presets
- Falsification **J**, **J′**

## Honest limits

- Does not prove a smooth manifold — only that MI rankings are persistent.
- High drift means magnitudes change; use ρ for structural stability.
- 2×2×2 cube ground dim remains inconclusive (see [universe-lab.md](./universe-lab.md)).
