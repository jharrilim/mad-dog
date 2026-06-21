# Spectrum-driven factorization search

Open hypothesis from [open-questions.md](./open-questions.md): given spectral data and/or a Hamiltonian, search for a tensor-factor labeling that makes physics look **local** on a graph.

## Modes (honest naming)

| Mode | What the search sees | Scorer |
|------|----------------------|--------|
| **Pauli + MI** (`inputMode: 'pauli'`) | Pauli expansion of Ĥ + low-energy states | H locality + multi-state MI + emergent dim |
| **Spectrum only** (`inputMode: 'spectrum'`) | Eigenvalues + eigenvector amplitudes only | MI locality + bandwidth proxy (no Pauli terms) |

## What the code does

1. Build a Hamiltonian — **shuffled chain/grid**, or **random non-local**.
2. Find `k` low-energy states (Gram–Schmidt deflation) or diagonalize for spectrum mode.
3. Search permutations maximizing locality on a **line** or **grid** graph.
4. Return baseline vs best MI heatmaps, coupling graph, and top-k candidates.

**Search methods:** `exact` (n≤8), `annealing` (default n>8), `greedy`.

## API

```ts
import { runFactorizationSearchAsync } from '@/sim/runner-async'

const result = await runFactorizationSearchAsync({
  kind: 'shuffled_chain',
  n: 6,
  field: 1.5,
  seed: 4242,
  inputMode: 'pauli',
  eigenstateCount: 3,
  searchMethod: 'exact',
})
```

Joint quench study (links to [refinement.md](./refinement.md)):

```ts
import { runFactorizationRefinementStudyAsync } from '@/sim/runner-async'

const study = await runFactorizationRefinementStudyAsync({
  n: 10, field: 1.5, dt: 0.2, steps: 14, seed: 7711,
})
```

WASM: `factorizationSearch`, `factorizationRefinement`.

## UI

- **Experiments** → *Locality from H and low-energy states* (`FactorizationViz`)
- **Experiments** → *Quench + factorization drift* (`FactorizationRefinementViz`)

## Limits

- Line/grid permutations only — no dynamic factor splitting.
- Spectrum mode is a **toy**: eigenvectors in the hidden computational basis, not true “only {Eₙ}” inference.
- `n > 8` uses simulated annealing (tune via `annealingSteps`).

## Checks

- `npm run check:wasm` — WASM smoke + structural invariants
- `npm run bench:factorization` — recovery matrix (WASM-only; n=6,8,10 + spectrum)

All calculations run in Rust/WASM; there is no TypeScript fallback.
