# Universe Lab (3+1)

## Goal

A separate page (`/lab`) showing **three emergent spatial dimensions + one emergent time** in one dashboard.

## Model

`tfimCube(lx, ly, lz)` — 3D TFIM with open boundaries.

- Index: `idx(x,y,z) = z·(lx·ly) + y·lx + x`
- Couplings along ±x, ±y, ±z
- `truePositions` with `{x,y,z}` for Procrustes alignment and edge drawing

## Presets

| Lattice | Qubits | Notes |
|---------|--------|-------|
| 2×2×2 (default) | 8 | Fast; ground-state dim inconclusive |
| 2×2×3 | 12 | **dim=3 benchmark** (ground + quench); slower |

**Clock slices (UI):** 12–80 on 2×2×2, 12–56 on 2×2×3; default 36 slices at Δt=0.25 (T≈9). Increase slices to probe late-time convergence of emergent geometry and defect signal.

3×3×3 (27 qubits) remains out of scope for the reference engine.

## Runner: `runUniverse3D`

1. Central defect quench on cube.
2. `buildSpacetime({ embedDim: 3, alignTo: truePositions })`.
3. Light cone with **Manhattan distance** on true lattice (not emergent coords).
4. MI for selected slice only (`stateAtUniverseSlice`) — performance.

## Visualization

| Element | Data |
|---------|------|
| Node position | MDS coords (3D), globally scaled |
| Node size/glow | `signal` (disturbance) |
| Edge opacity | MI between neighbours at selected slice |
| Purple trail | Defect worldline in emergent coords, truncated at selected `k` |
| Heatmap | sites × time, signal intensity |
| WebGL | Three.js / react-three-fiber, OrbitControls |

Time is **not** a fourth graphics axis — see [emergent-time.md](./emergent-time.md).

## Measured results (`sim-check`)

**Ground state 2×2×2, h=1.5:**

- emergentDim = 1 (inconclusive at n=8)

**Ground state 2×2×3, h=1.5:**

- emergentDim = 3 ✓

**Quench mid-slice (k=3), h=1.5:**

- 2×2×2 and 2×2×3: emergentDim = 3 ✓

**Quench 3+1:**

- Coords length 3 per site ✓
- Energy drift ~ 10⁻⁸ ✓
- Finite LR velocity ✓

## UI gotchas fixed

- Canvas needs **fixed height** (`h-[min(420px,55vh)]`), not `h-full` in an unconstrained grid — otherwise layout can grow unbounded.
- Block `wheel` events on canvas container so OrbitControls zoom doesn't scroll the page.

## Honest disclaimer (shown on page)

- 8-qubit toy universe.
- Positions reconstructed from entanglement.
- Emergent dim ≈ 3 is **testable at 2×2×3** with the improved estimator; 2×2×2 remains inconclusive on ground states.
