# Spectrum-driven factorization search (prototype)

Open hypothesis from [open-questions.md](./open-questions.md): given a Hamiltonian (or its spectrum and ground state), search for a tensor-factor labeling that makes Ĥ look **local** on a line graph.

## What the code does

1. Build a Hamiltonian — either a **shuffled TFIM chain** (known-local H with permuted qubit labels) or a **random non-local** model.
2. Find the ground state (imaginary-time descent).
3. Search permutations `perm[q] = line_position` that maximize a combined score:
   - **H locality** — fraction of two-body terms with `|perm[i] − perm[j]| = 1`
   - **MI locality** — average mutual information on line neighbours vs farther pairs on the ground state
4. Return baseline (identity labeling), best permutation, and top-k candidates.

For `n ≤ 8` the search enumerates all permutations; for larger `n` it uses random restarts plus greedy pairwise swaps.

## API

```ts
import { runFactorizationSearchAsync } from '@/sim/runner-async'

const result = await runFactorizationSearchAsync({
  kind: 'shuffled_chain', // or 'random'
  n: 6,
  field: 1.5,
  seed: 4242,
  topK: 5,
})
```

WASM method: `factorizationSearch` → `run_factorization_search_json`.

## UI

**Experiments** → *Locality from the spectrum* (`FactorizationViz`).

## Limits (honest)

- Only **line** factorizations (1D nearest-neighbour graph), not general graphs or growing factor count.
- Scoring uses the **ground state**, not full spectrum data.
- `n > 8` search is heuristic, not exhaustive.
- Recovering a shuffle proves the search works; random non-local H may have no good line factorization.

## Checks

`npm run check:wasm` compares TS reference vs WASM on a shuffled 6-site chain.
