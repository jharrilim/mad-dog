# Adaptive holographic refinement

Speculative extension of [open-questions.md](./open-questions.md#mad-dog-native-extension-adaptive-holographic-refinement): factor count is not primitive but the size of the *minimal* local factorization that keeps holographic-style diagnostics healthy for the current |ψ⟩.

## Pressure score (0–1)

`wasm/mad-dog-sim/src/refinement.rs` combines:

| Ingredient | Source | Failure mode |
|------------|--------|--------------|
| RT fit | `analyze_rt_relation` | Low R² |
| RT slope | same | \|slope − 1\| large |
| Area-law proxy | S(n/2)/S(1) vs ground/random baselines | Volume-like growth |
| Emergent dimension | `analyze_emergent_geometry` | MI geometry too high-D |

`needs_refinement` fires when composite pressure or paired reason flags cross hand-tuned thresholds (`DEFAULT_REFINEMENT_THRESHOLDS`).

## Dynamical split heuristic (2026-06)

**Trigger:** peak-pressure step among a quench that ends with `needs_refinement = true` (avoids early false triggers before entanglement builds).

**Split rule (toy):** at trigger depth, compare diagnostics on chain `n` vs chain `n+Δn` (same defect quench, same step count). **Accept** if `pressure(n+Δ) < pressure(n)`.

**Split site hint:** `suggest_split_site` — lattice index where edge-anchored entropy grows fastest (not a real tensor factorization).

WASM: `run_adaptive_refinement_json` → `AdaptiveRefinementResult` with `splitEvent`.

```ts
import { runAdaptiveRefinementAsync } from '@/sim/runner-async'

const result = await runAdaptiveRefinementAsync({
  n: 10, field: 1.5, dt: 0.2, steps: 18, seed: 7711, deltaN: 2,
})
// result.splitEvent: { triggerStep, pre, post, accepted, pressureDelta, ... }
```

UI: **Experiments** → *Adaptive holographic refinement* (`AdaptiveRefinementViz`)

CLI sweep: `npm run sweep:refinement` → `scripts/refinement-adaptive.ts`

Falsification test **H**: split triggers and is accepted on default TFIM demo.

## Earlier prototypes (still available)

| Tool | Purpose |
|------|---------|
| `runRefinementQuenchAsync` | Pressure trace over quench |
| `runRefinementNCompareAsync` | Manual n vs n+Δ at fixed step |
| `runFactorizationRefinementStudyAsync` | Joint factorization drift + pressure |

## Honest limits

- **No in-place split** — we compare independent quenches on different chain lengths, not embedding |ψ⟩ into a larger Hilbert space.
- **No tensor-network bond truncation** — adding factors means `tfimChain(n+Δ)`, not splitting a qubit.
- Thresholds tuned on 10-site gapped chains. Accept/reject is a diagnostic hook, not a theorem.

## Typical result (n=10, h=1.5, steps=18, Δ=2)

- Trigger mid-quench when RT/area components decouple
- `accepted=true`, Δpressure ≈ 0.05–0.15 (larger chain relieves stress)
- See also [factorization.md](./factorization.md) for labeling drift under the same quench
