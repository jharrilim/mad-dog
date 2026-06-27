# Spectrum-driven factorization search

Open hypothesis from [open-questions.md](./open-questions.md): given spectral data and/or a Hamiltonian, search for a tensor-factor labeling that makes physics look **local** on a graph.

## Modes (honest naming)

| Mode | What the search sees | Scorer |
|------|----------------------|--------|
| **Pauli + MI** (`inputMode: 'pauli'`) | Pauli expansion of Ĥ + low-energy states | H locality + multi-state MI + emergent dim |
| **Spectrum only** (`inputMode: 'spectrum'`) | Eigenvalues + eigenvector amplitudes only | Weighted MI + **line-support bandwidth** (permutation-aware span of active sites) + emergent dim |

Spectrum scoring uses low-lying eigenstate weights (heavier weight on ground state) and measures how compact each eigenvector’s support is along the **candidate line** after permuting qubit indices — not raw Hamming weight, which ignores the permutation. On an open chain, the reflected labeling (k ↦ n−1−k) is equivalent; recovery uses `line_equiv_distance`.

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

Run `npm run bench:factorization` for the full matrix plus Phase-1 ensemble (`run_factorization_ensemble_json`). Falsification **K** (≥80% ensemble) and **M** (negative controls) in [falsification.md](./falsification.md).

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

## Limits

- Line/grid/torus/cube permutations — no dynamic factor splitting.
- Spectrum mode is a **toy**: eigenvectors in the hidden computational basis, not true “only {Eₙ}” inference.
- ~~Torus/cube builders exist but are not factorization targets yet.~~ **Shipped (AH)** — `GraphKind::Torus` / `Cube`; `shuffled_torus` (2×2) and `shuffled_cube` (2×2×2) in factorization search.
- `n > 8` uses simulated annealing (tune via `annealingSteps`).

## Checks

- `npm run check:wasm` — WASM smoke + structural invariants
- `npm run bench:factorization` — recovery matrix + ensemble
- `npm run check:falsification` — battery **K**, **M**

All calculations run in Rust/WASM; there is no TypeScript fallback.
