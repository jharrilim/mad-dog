# Spectrum-driven factorization search

Open hypothesis from [open-questions.md](./open-questions.md): given spectral data and/or a Hamiltonian, search for a tensor-factor labeling that makes physics look **local** on a graph.

## Modes (honest naming)

| Mode | What the search sees | Scorer |
|------|----------------------|--------|
| **Pauli + MI** (`inputMode: 'pauli'`) | Pauli expansion of Ĥ + low-energy states | H locality + multi-state MI + emergent dim |
| **Spectrum only** (`inputMode: 'spectrum'`) | Eigenvalues + eigenvector amplitudes only | Weighted MI + **graph support bandwidth** (permutation-aware span/diameter of active sites on the candidate graph) + emergent dim |
| **Eigenvalues only** (`inputMode: 'eigenvaluesOnly'`) | Low-lying {Eₙ} only — **cannot recover labeling** | Permutation-invariant level-spacing summary (flat across all candidates) |

Spectrum scoring uses low-lying eigenstate weights (heavier weight on ground state) and measures how compact each eigenvector’s support is on the **candidate graph** after permuting qubit indices — line uses index span; grid/torus use Manhattan/torus graph diameter among active sites. On an open chain, the reflected labeling (k ↦ n−1−k) is equivalent; recovery uses `line_equiv_distance`. On square **grid/torus**, the eight D₄ rigid motions (rotations + reflections) are equivalent; recovery uses `grid_equiv_match` / `lattice_equiv_match`.

## Model zoo (Phase 1, 2026-06)

| `kind` | Builder | Blind shuffle |
|--------|---------|---------------|
| `shuffled_chain` | TFIM | yes |
| `shuffled_xx_chain` | XX + transverse Z | yes |
| `shuffled_heisenberg_chain` | XXX+YYY+ZZZ | yes |
| `shuffled_sparse_chain` | Random NN Pauli subset | yes |
| `shuffled_grid` | TFIM grid | yes |
| `shuffled_torus` | TFIM torus (periodic) | yes |
| `shuffled_cube` | TFIM cube (open 3D) | yes |
| `random` | `random_nonlocal` | no ground truth |

## Recovery benchmarks

| Case | Spectrum recovery |
|------|-------------------|
| TFIM shuffled n=4–8 | ✓ |
| XX / Sparse n=6 | ✓ spectrum |
| Heisenberg n=6 | ✓ Pauli+MI in ensemble (spectrum scorer weak) |
| scrambled spectrum (negative) | ✗ (by design) |
| random n=6 | no ground truth |

Run `npm run bench:factorization` for the full matrix plus Phase-1 ensemble (`run_factorization_ensemble_json`). Falsification **K** (≥80% ensemble) and **M** (negative controls at n=6 and n=8) in [falsification.md](./falsification.md). `npm run check:scaling` asserts uniqueness metrics across the model zoo (TFIM n=4/6/8, XX n=6, sparse n=6).

## Uniqueness report

When ground-truth shuffle is known, results include `uniqueness`:

- `equivalenceClassCount` — clusters in top-k by line reflection equivalence
- `bestClassSize` — size of highest-scoring class (2 = reflection degeneracy only)
- `trueInTopK`, `trueClassRank`, `scoreGapToSecondClass`

## Negative control: `spectrumScramble`

Set `spectrumScramble: true` to permute qubit labels in eigenvector amplitudes before scoring (eigenvalues unchanged). Recovery should fail — used in falsification **M**.

## What the code does

1. Build a Hamiltonian — shuffled local model or random non-local.
2. Find `k` low-energy states or diagonalize for spectrum mode.
3. Search permutations maximizing locality on a **line**, **grid**, **torus**, or **cube** graph.
4. Return baseline vs best MI heatmaps, coupling graph, top-k candidates, uniqueness.

**Search methods:** `exact` (n≤8), `annealing` (default n>8), `greedy`.

## API

```ts
import { runFactorizationSearchAsync } from '@/sim/runner-async'

const result = await runFactorizationSearchAsync({
  kind: 'shuffled_heisenberg_chain',
  n: 6,
  field: 1.5,
  seed: 4242,
  inputMode: 'spectrum',
  eigenstateCount: 4,
  searchMethod: 'exact',
})
```

Ensemble (Phase 1 battery):

```ts
// scripts/factorization-bench.ts calls WASM directly:
// run_factorization_ensemble_json('{}')
```

WASM: `factorizationSearch`, `factorizationRefinement`, `factorizationEnsemble`.

## UI

- **Experiments** → *Locality from H and low-energy states* (`FactorizationViz`) — model zoo buttons + uniqueness panel
- **Experiments** → *Quench + factorization drift* (`FactorizationRefinementViz`)

## Eigenvalue-only impossibility (shipped 2026-06-27)

Global Ĥ eigenvalues are **invariant under qubit permutations** — no labeling information is present in {Eₙ} alone. The codebase wires `inputMode: 'eigenvaluesOnly'` as an explicit negative demo; falsification **AK** checks that recovery fails while spectrum+ψ control still succeeds.

WASM battery: `run_eigenvalue_only_json`

## AA-blind 2D recovery (shipped 2026-06-27)

**AA-blind** = `spectrum_scrambled=true` (no Ĥ term): 50% MI-NN ratio + 35% bandwidth + 15% emergent-dim bonus. Falsification **AA** (chain) and **AL** (2D) use this path via `locality_spectrum.rs`.

| System | seed=4242 | Notes |
|--------|-----------|-------|
| chain n=4–8 | ✓ | unchanged |
| grid 3×3 | ✓ | true labeling in top score bucket (8-way D₄ tie); recovery via `grid_equiv_match` |
| torus 2×2 | ✓ | `blind_2d_mi_term` penalizes spurious MI-NN > 1 (wrap graph has no distance-≥2 pairs for far-MI contrast) |

`npm run check:scaling` asserts grid/torus AA-blind recovery; falsification **AL** passes when chain + grid + torus all recover.

## Gapless / frustrated blind limits (shipped 2026-06-27)

**AA-blind** on gapless XX chain (h/J < 1, e.g. h=0.5, J=1) **cannot** recover labeling: mutual information decays algebraically with distance, so NN vs far-MI contrast vanishes and permutations score identically (top-20 spread < 0.02). Falsification **AN** and `check:scaling` `aa_blind_xx_gapless` encode this as a **correct negative**.

| Mode | XX n=6 h=0.5 | Notes |
|------|--------------|-------|
| Semi-blind spectrum (40% Ĥ + MI) | ✓ | Hamiltonian locality term breaks degeneracy |
| AA-blind (MI+bandwidth only) | ✗ | Flat landscape — fundamental for this scorer |
| Pauli + MI | ✓ | Full Ĥ structure available |

Fixing AA-blind on gapless models requires **minimal extra data** beyond {Eₙ, ψₙ} MI+bandwidth — e.g. low-weight Pauli expectations or correlators (roadmap Priority 2 open item).

## Limits

- Line/grid/torus/cube permutations — no dynamic factor splitting.
- ~~Spectrum mode is a **toy**: eigenvectors in the hidden computational basis, not true “only {Eₙ}” inference.~~ **Eigenvalues-only mode shipped** as documented impossibility; spectrum mode still needs ψₙ amplitudes.
- ~~Torus/cube builders exist but are not factorization targets yet.~~ **Shipped (AH)** — `GraphKind::Torus` / `Cube`; `shuffled_torus` (2×2) and `shuffled_cube` (2×2×2) in factorization search.
- `n > 8` uses simulated annealing (tune via `annealingSteps`).

## Checks

- `npm run check:wasm` — WASM smoke + structural invariants
- `npm run bench:factorization` — recovery matrix + ensemble
- `npm run check:falsification` — battery **K**, **M**
- `npm run check:scaling` — Phase 12 scaling + uniqueness zoo

All calculations run in Rust/WASM; there is no TypeScript fallback.
