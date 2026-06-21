# Emergent Space

## Pipeline

1. **Single-site entropy** `S_a` — partial trace to one qubit.
2. **Two-site entropy** `S_{ab}` — partial trace to pairs.
3. **Mutual information** `I(a:b) = S_a + S_b - S_{ab}` (clamped ≥ 0).
4. **Distance** `d(a,b) = -ξ ln(I(a:b) / I_max)`; negligible MI → capped distance.
5. **Classical MDS** on the distance matrix → embedding coords + Gram eigenvalues.
6. **Emergent dimension** — largest relative gap in the positive eigenvalue spectrum (restricted to eigenvalues > 1% of max).

## What we expect (and verified)

| Model | Expected dim | `sim-check` observation |
|-------|--------------|-------------------------|
| TFIM chain, h=1.5 | 1 | emergentDim = 1, NN MI > far MI |
| TFIM grid 3×3, h=1.5 | 2 | emergentDim = 2 |
| Random non-local | high | emergentDim ≈ 3+, messy spectrum |
| TFIM cube 2×2×2, h=1.5 | 3 | emergentDim = 2 (noisy at n=8) |

**Lesson:** With only 8 points, 3D dimension detection is unreliable. The MDS may carry three coordinates, but the gap heuristic often reports 2. This is an honest limitation of the minimal cube — not proof against 3D emergence at larger scale.

## Gauge freedom

MDS has rotation/reflection symmetry. For time-evolving 2D/3D embeddings we **Procrustes-align** each slice to the model's `truePositions` (known lattice layout used only for visualization stability, not fed to the physics).

- `procrustes2D` — chain/grid spacetime demos
- `procrustes3D` — Universe Lab

Recovered positions are **reconstructions from entanglement**, not read off a built-in coordinate chart.

## When geometry is clean vs messy

**Paramagnetic phase (h ≳ 1):** MI decays with graph distance → clean low-D MDS for **ground states**. This is why emergence demos use h = 1.0–1.5.

**Ordered phase (h ≲ 1):** long-range Z correlations → MI less local, MDS geometry less faithful to true lattice (but quench dynamics can look sharper — see [parameters-and-phases.md](./parameters-and-phases.md)).
