# Falsification Criteria

What would **disprove** Mad-Dog-specific emergence claims — as opposed to generic quantum-information facts (area laws in 1D gapped systems, etc.).

## Generic vs specific

| Observation | Generic QI? | Mad-Dog-specific? |
|-------------|-------------|-------------------|
| Ground-state area law | Yes (gapped 1D) | No |
| RT slope ≈ 1 on ground | Approximate for special states | Partially |
| Emergent dim = lattice dim (TFIM chosen to match) | Circular if model known | Weak |
| RT slope ↑ under excitation | Expected if RT is class-specific | No (failure of identity, not curvature) |

## Automated battery

Run: `npm run check:falsification` or **Experiments → Falsification Tests**.

| ID | Claim tested | Pass criterion |
|----|--------------|----------------|
| **A** | Blind locality recovery | Shuffled chain n=8 recovers identity permutation |
| **A′** | Cross-graph structure | Open grid: line ≈ grid score; torus: line locality < 90% |
| **B** | MI geometry faithful | \|v_MI / v_lattice − 1\| < 0.25 |
| **C** | RT ratio on ground | std(S_A / ½ boundary MI) < 0.15 across intervals |
| **D** | Modular clocks desync | In-cone modular sync R² < 0.95 |
| **E** | Excitations propagate | Both worldlines move; min separation ≥ 1 during overlap |
| **F** | Refinement decoupling | Slope vs area diagnostic peaks diverge under quench |

## Implementation

- Rust: [`wasm/mad-dog-sim/src/run_falsification.rs`](../wasm/mad-dog-sim/src/run_falsification.rs) → `run_falsification_battery_json`
- CLI: `npm run check:falsification` → [`scripts/falsification-check.ts`](../scripts/falsification-check.ts)
- UI: [`FalsificationViz`](../src/components/FalsificationViz.tsx) (async WASM worker)

Thresholds are hand-tuned on TFIM demos; treat failures as research signals, not theorem violations.
