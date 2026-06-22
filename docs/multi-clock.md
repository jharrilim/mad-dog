# Multi-clock consistency networks

**Shipped 2026-06** — extends [emergent-time.md](./emergent-time.md) relational clocks to N sites.

## Setup

One central-defect quench, one uniform Δt clock, and N **physical clocks** at chosen sites. Each physical clock ticks when local signal `|⟨Z⟩ − reference|` crosses evenly spaced thresholds (same rule as the two-clock demo).

At each uniform slice k, each clock assigns an interpolated **physical tick reading**. Pairwise **sync R²** fits affine maps between tick readings on the shared trajectory.

## What we measure

| Metric | Meaning |
|--------|---------|
| `pairwiseR2[i][j]` | Do clocks i and j agree on time labels for the same states? |
| `defectUniformR2` | Defect-site clock vs uniform (expect ≈ 1) |
| `edgeEdgeR2` | Left edge vs right edge (expect < 1 mid-quench) |
| `minPairwiseR2` | Weakest link in the network |
| `inconsistentPairs` | Count of pairs with R² < 0.95 |

## Typical results (n=9, h=1, sites 0 / 4 / 8)

- Defect ↔ uniform: R² ≈ 1.0
- Edge ↔ uniform: R² ≈ 0.8 (stall then catch-up)
- Edge ↔ edge: R² < 0.95 — **no global time coordinate**

Interpretation: clocks local to the disturbance stay synced with the uniform discretization; distant clocks disagree with each other even when each partially tracks the uniform clock. Time is relational, not a unique global parameter.

## API

WASM: `run_multi_clock_json` → `MultiClockResult`

```ts
import { runMultiClockAsync } from '@/sim/runner-async'

const result = await runMultiClockAsync({
  kind: 'grid',
  rows: 3,
  cols: 3,
  field: 1,
  dt: 0.2,
  steps: 40,
  physicalSlices: 15,
})
```

**G′** (grid) uses the same pass criterion as **G**. Cube 2×2×3 covered in `npm run bench:time`.

UI: **Experiments** → *Multi-clock consistency network*

Falsification test **G**: defectUniformR² > 0.95 **and** minPairwiseR² < 0.95.

## Limits

- Z-signal threshold clocks only (modular-flow network on lattices not yet wired).
- ~~1D chain; cube multi-clock not wired.~~ **2026-06 Phase 3:** grid 3×3 and cube 2×2×3 multi-clock; simultaneity bend metric **N**.
- R² threshold 0.95 is hand-tuned on TFIM demos.
