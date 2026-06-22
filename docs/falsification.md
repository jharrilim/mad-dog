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
| **G** | Multi-clock not globally consistent | defectUniformR² > 0.95 and minPairwiseR² < 0.95 |
| **H** | Adaptive split relieves pressure | Split triggers and accepted on default quench |
| **I** | Branch excitations code-like | Sharpness gain > 1.03, window rank drops, branch overlap < 0.95 |
| **J** | Gauge-free MI geometry stable | Ordered phase mean Spearman ρ > 0.85 |
| **K** | Ensemble blind locality recovery | Phase-1 model zoo: ≥80% recovery (spectrum; Pauli for Heisenberg/sparse) |
| **L** | Directional causal isotropy (Lorentz proxy v0) | Cardinal front-speed CoV < 0.25 on 3×3 grid quench |
| **M** | Negative controls reject fake locality | `random` n=6 and scrambled-spectrum shuffled chain do not recover |

**K**, **L**, and **M** target [roadmap.md](./roadmap.md) signatures **S1** and **S9**. **L** is a stub — extend to n-scaling and dispersion before treating as a strong Lorentz claim.

## Planned extensions

| ID | Signature | Target criterion (not yet in battery) |
|----|-----------|----------------------------------------|
| **J′** | S2 — 3D gauge-free geometry | 2×2×3 quench mean Spearman ρ > 0.85 |
| **I′** | S8 — stabilizer identification | Pauli generating set on branch subspace |

## Implementation

- Rust: [`wasm/mad-dog-sim/src/run_falsification.rs`](../wasm/mad-dog-sim/src/run_falsification.rs) → `run_falsification_battery_json`
- Ensemble: [`wasm/mad-dog-sim/src/run_factorization_ensemble.rs`](../wasm/mad-dog-sim/src/run_factorization_ensemble.rs) → `run_factorization_ensemble_json`
- CLI: `npm run check:falsification` → [`scripts/falsification-check.ts`](../scripts/falsification-check.ts)
- UI: [`FalsificationViz`](../src/components/FalsificationViz.tsx) (async WASM worker)

Thresholds are hand-tuned on TFIM demos; treat failures as research signals, not theorem violations.
