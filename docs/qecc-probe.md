# Excitation subspace probe (QECC / EFT)

**Status:** prototype (2026-06). Not a claim that TFIM excitations form a literal QECC — a diagnostic for whether branch-resolved defects look like a low-rank, Pauli-biased subspace after decoherence.

## Motivation

Carroll–Singh open question: are infrared matter degrees of freedom a **code subspace**? After our minimal environment coupling ([decoherence.md](./decoherence.md)), each Everett branch should pin the defect to a sharper classical configuration. This probe asks whether that sharpening also shows up in **entanglement structure** on the excitation window.

## Model

Same as decoherence quench: TFIM chain + 1 env qubit, central defect, CNOT + ZX coupling at `coupleStep`. At the final slice we:

1. Partial-trace the environment → chain marginal (mixed) and conditional branches |ψ₀⟩, |ψ₁⟩.
2. Pick an excitation window around the peak of |⟨Z⟩| on the mixed marginal.
3. Compare **Pauli sharpness** (mean |⟨X⟩|, |⟨Z⟩| on window sites) and **effective rank** of the window RDM.

## Metrics

| Metric | Meaning |
|--------|---------|
| `sharpnessGain` | Branch avg sharpness / mixed sharpness (>1 ⇒ branches more Pauli-biased) |
| `rankReduction` | Branch window effective rank / mixed window rank (<1 ⇒ branches lower entropy) |
| `branchOverlap` | \|⟨ψ₀\|ψ₁⟩\|² on chain (should be small when branches differ) |
| `codeLike` | Heuristic pass: both branches weighted, gain > 1.03, rank reduction < 0.98, overlap < 0.95 |

## API

```typescript
import { runExcitationSubspaceAsync } from '@/sim/runner-async'

const result = await runExcitationSubspaceAsync({
  n: 8,
  field: 1.2,
  dt: 0.2,
  steps: 22,
  coupleStep: 7,
  coupling: 0.9,
  windowRadius: 2,
})
```

WASM: `run_excitation_subspace_json` in `excitation_subspace.rs`.

## Measured (default config)

With n=8, h=1.2, coupling=0.9: branches sharpen (gain ≈ 1.09), rank drops on the window, overlap ≈ 0 — **code-like** in the heuristic sense.

## UI / falsification

- **Experiments** → *Excitation subspace probe*
- Falsification test **I**: branch excitations sharpen into low-rank Pauli-biased subspace

## Honest limits

- Window size is hand-picked; no automatic stabilizer search in the Pauli group.
- “Code-like” ≠ identified [[n,k,d]] code; it is a proto-EFT sanity check.
- Rank comparison is on a small window only (full chain marginal can remain high-rank).
