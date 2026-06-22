# Gauge-free MI geometry stability

**Status:** prototype (2026-06). Measures whether **mutual-information distance matrices** stay structurally consistent across clock slices — without Procrustes alignment or `truePositions`.

## Motivation

Emergent MDS coordinates require gauge choices (rotation, reflection). The MI-derived **distance matrix** D_ij is label-invariant: it depends only on qubit indices and the state. If “space” is real, relative distance **rankings** should persist as a defect propagates (even when absolute distances drift).

## Setup

Central defect quench on TFIM chain (ordered phase h ≈ 1.2). At each slice k:

1. Build MI matrix → distance matrix D(k) via `mi_to_distance`.
2. Compare D(k) to D(k−1):
   - **Relative Frobenius drift** ‖D(k) − D(k−1)‖ / ‖D(k−1)‖
   - **Spearman ρ** on off-diagonal entries (gauge-free)
3. Record emergent dimension and top Gram eigenvalues (also gauge-free).

## Pass criteria

`geometryStable` ⇔ mean Spearman ρ > 0.85 across slices.

Absolute distance drift can be large during quench propagation; rank correlation is the primary invariant.

Ordered phase (h=1.2) passes mean ρ > 0.85 in falsification test **J** (paramagnetic can also rank highly — contrast not required).

## API

```typescript
import { runGeometryStabilityAsync } from '@/sim/runner-async'

const result = await runGeometryStabilityAsync({
  n: 10,
  field: 1.2,
  dt: 0.2,
  steps: 20,
})
```

WASM: `run_geometry_stability_json` in `geometry_stability.rs`.

## Measured (default config)

n=10, h=1.2, 20 steps: mean ρ ≈ 0.93, drift ≈ 0.45 (expected during propagation), dim σ ≈ 0.73.

## UI / falsification

- **Experiments** → *Gauge-free MI geometry stability*
- Falsification test **J**: ordered-phase ranking stable vs paramagnetic contrast

## Honest limits

- 1D chain only; 2D/3D stability not yet scanned.
- Does not prove a smooth manifold — only that MI rankings are persistent.
- High drift means magnitudes change; use ρ for structural stability.
