# Adaptive holographic refinement (prototype)

Speculative extension of [open-questions.md](./open-questions.md#mad-dog-native-extension-adaptive-holographic-refinement): factor count is not primitive but the size of the *minimal* local factorization that keeps holographic-style diagnostics healthy for the current |ψ⟩.

## What we built

`wasm/mad-dog-sim/src/refinement.rs` implements a **refinement pressure** score (0–1) from:

| Ingredient | Source | Failure mode |
|------------|--------|--------------|
| RT fit | `analyzeRtRelation` | Low R² — entropy does not track boundary “area” |
| RT slope | same | |slope − 1| large — discrete RT relation deformed |
| Area-law proxy | S(n/2)/S(1) vs ground/random baselines | Volume-like growth under quench |
| Emergent dimension | `analyzeEmergentGeometry` | MI geometry higher-dimensional than expected |

**Quench study:** `runRefinementQuench` evolves a central defect under the TFIM chain and records pressure per clock step.

**n comparison:** `runRefinementNCompare` runs the same quench depth at `n` and `n+Δn` and asks whether the larger chain lowers pressure (toy “add factors” test).

UI: `RefinementViz` on the essay page (holographic section).

CLI: `scripts/sim-check.ts` block prints vacuum vs late pressure and n vs n+2 comparison.

**WASM:** `runRefinementQuenchAsync` / `runRefinementNCompareAsync` in `runner-async.ts` (validated in `scripts/wasm-check.ts`).

See also [factorization.md](./factorization.md) — joint quench + factorization drift study (`FactorizationRefinementViz`).

## What we did *not* build

- No dynamic tensor-network splitting or bond-dimension truncation search.
- No spectrum-driven factorization search.

## Honest limits

Thresholds in `DEFAULT_REFINEMENT_THRESHOLDS` are hand-tuned on 10-site gapped chains. The n+2 comparison is suggestive, not a proof that “the universe added a qubit.” Use as a diagnostic hook for further experiments.
