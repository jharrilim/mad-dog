# Holography Tests

## Motivation

If entanglement builds emergent geometry, a nearby bold hypothesis is **holography**: region entropy is set by **boundary** data, not bulk volume.

We test two discrete analogues on TFIM chains (honest attempts, not AdS/CFT claims).

## 1. Area law vs volume law

For nested regions A (edge-anchored intervals):

| State | S(A) scaling | `sim-check` (n=10, h=1.5) |
|-------|--------------|---------------------------|
| Gapped ground | **Area law** — saturates ~0.15 | flat vs \|A\| |
| Haar random | **Volume law** — Page curve | grows ~ \|A\|·ln2 to half-chain |

Ground: `S(|A|=5) ≈ 0.15` vs random `≈ 2.98` → **OK (area ≪ volume)**.

## 2. Baby Ryu–Takayanagi relation

Carroll–Singh (building on Cao–Carroll–Michalakis) suggest for redundancy-constrained states:

```
S_A  ≈  (1/2) Σ_{a∈A, b∉A}  I(a:b)
```

The RHS is entanglement crossing the boundary — discrete "area."

We fit `S_A = slope · (boundary cut)` through the origin for all contiguous intervals.

| Condition | Slope | R² |
|-----------|-------|-----|
| Vacuum ground (h=1.5) | 0.94 | 0.993 |

Slope ≈ 1 is what the essay predicts. Relation holds strongly in the gapped ground state.

## 3. Mass deforming the RT slope

"Mass" = local X-flip at centre + brief evolution under H (concentrated entanglement).

| Mass strength | RT slope | R² |
|---------------|----------|-----|
| 0 (vacuum) | 0.94 | 0.993 |
| 1.0 (quench time) | 1.40 | 0.929 |

Slope **tilts upward**; linear fit degrades but does not collapse.

**Honest interpretation:** the discrete RT identity \(S_A \approx \frac{1}{2}\sum I(a{:}b)\) holds for redundancy-constrained states, not arbitrary excitations. Slope \(\approx 1.4\) means the state left that class — the formula broke down — not that we measured emergent scalar curvature. See [falsification.md](./falsification.md).

## 4. Multi-insertion / excitation density (2026-06)

At fixed quench strength (`strength=1`), evenly spaced X-flips across the inner chain:

| # excitations | ρ = count/n | RT slope | Δslope vs vacuum |
|---------------|-------------|----------|------------------|
| 1 | 0.10 | ~1.40 | ~+0.46 |
| 2 | 0.20 | ~1.47 | ~+0.53 |
| 3 | 0.30 | ~2.17 | ~+1.22 |
| 4 | 0.40 | ~2.35 | ~+1.40 |
| 5 | 0.50 | ~2.28 | ~+1.34 |

Run `npm run sweep:mass` for live numbers. **Δslope grows with ρ** (roughly linear at low ρ; RT fit degrades at ρ≳0.4 as R² drops).

WASM: `inject_mass_multi`, `densitySweep` on `run_rt_mass_json`. UI: third chart on *Mass Curves the RT Slope* (Experiments).

## Implementation notes

- `entropyOfRegion` — partial trace on smaller side (S_A = S_Ā for pure states).
- RT points: all contiguous intervals on the chain.
- Mass injection: `kickX` + `evolveInterval` for strength × evolution time.
