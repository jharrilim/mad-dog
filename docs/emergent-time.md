# Emergent Time

## Page–Wootters mechanism

No external time parameter. We construct a discrete history:

```
|Ψ⟩ ∝ Σ_k |k⟩_clock ⊗ e^{-iH t_k} |ψ₀⟩
```

Slice `k` is the **conditioned** system state at emergent time `t_k = k · Δt`.

Implementation: evolve `|ψ₀⟩` under `H` for `k` steps; at each step run the geometry pipeline. Each slice is one "moment" of emergent history.

## Quench protocol

Standard for spacetime demos:

- **Initial:** product state |0…0⟩, except one flipped spin (defect) at centre.
- **Reference:** |0…0⟩ evolved in lockstep.
- **Signal:** `|⟨Z_i⟩_defect − ⟨Z_i⟩_reference|` — isolates propagating perturbation from uniform on-site precession.

Without reference subtraction, ⟨Z⟩ precession from the transverse field dominates every site and no light cone is visible.

## Light cone / Lieb–Robinson velocity

`measureLightCone` finds first arrival time of signal above threshold at each site, fits `distance = v · t` (Manhattan distance on lattice for cube).

`sim-check` (9-site chain): arrival times increase with distance from defect → finite speed. Energy drift ~ 10⁻⁸ per slice with adaptive substepping.

## How time is visualized (Universe Lab)

| UI element | Meaning |
|------------|---------|
| 3D canvas | **One spatial snapshot** at clock reading k |
| Slider / Play | Steps through k = 0, 1, 2, … (not motion inside a frame) |
| Heatmap | Full spacetime: qubits × emergent time; brightness = signal |
| Label `t = k·Δt` | Discrete emergent time coordinate |

This is honestly **3+1**: three emergent spatial coords (MDS) + one time axis on scrubber/heatmap — not a fourth graphics axis.

## Relational time (two clocks)

Same physical trajectory, two time readouts:

1. **Uniform clock:** fixed Δt ticks (global discretization).
2. **Physical clock:** ticks when disturbance signal at a chosen site crosses thresholds.

**Relational time map:** at physical tick τ_B, what uniform reading τ_A records the same state?

- Edge clock (far from defect): **desynchronizes** — R² ≈ 0.80; plateau then burst (stall then catch-up).
- Clock at defect site: **stays synchronized** — R² ≈ 1.0.

Neither clock is "the" time; they disagree when the local physics differs.

## Open issue

Three-or-more-clock networks and consistency conditions not yet explored.
