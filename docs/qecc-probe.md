# Excitation subspace probe + QECC code subspace identification

**Status:** shipped (2026-06). Two-stage pipeline:
1. **Excitation subspace probe** — branch sharpness / rank reduction on defect window.
2. **QECC code subspace identification** — explicit [[n,k,d]] label from commuting stabilizer generators + code subspace fidelity for both branches and the mixed state.

## Motivation

Carroll–Singh open question: are infrared matter degrees of freedom a **code subspace**? After our minimal environment coupling ([decoherence.md](./decoherence.md)), each Everett branch should pin the defect to a sharper classical configuration. This probe asks whether that sharpening also shows up in **entanglement structure** on the excitation window.

## Model

Same as decoherence quench: open chain + 1 env qubit, central defect, CNOT + ZX coupling at `coupleStep`. Set `model` to `tfim` (default), `xx`, or `heisenberg` for the bulk Hamiltonian.

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

## QECC code subspace identification

Given the generators {g₁,…,gₘ} found by the stabilizer search, the code subspace is their shared
eigenspace.  The fidelity of a state |ψ⟩ with this subspace is:

  F(|ψ⟩) = ∏ᵢ (1 + |⟨ψ|gᵢ|ψ⟩|) / 2

computed directly from the per-generator expectations already stored in the stabilizer search report.

| Metric | Meaning |
|--------|---------|
| `codeLabel` | `[[n,k,d]]` — n=window sites, k=n−generators, d=code distance |
| `branch0CodeFidelity` | ∏(1+\|⟨g⟩₀\|)/2 — branch 0 in code subspace |
| `branch1CodeFidelity` | ∏(1+\|⟨g⟩₁\|)/2 — branch 1 in code subspace |
| `mixedCodeFidelity`   | ∏(1+\|⟨g⟩_mixed\|)/2 — mixed state (pre-branch) in code |
| `fidelitySelectivity` | branch avg / mixed — >1 means code is branch-selective |
| `codeSubspaceFound`   | k ≥ 1, stabilizer found, branch avg fidelity > 0.5 |

**Measured (n=8, h=1.2, coupling=0.9, window_radius=2):** code subspace found, selectivity > 1.1,
meaning branches are more strongly inside the code than the pre-decoherence mixed state.

Falsification test **X**: `codeSubspaceFound && fidelitySelectivity > 1.1` (TFIM default).

**Non-TFIM (2026-06-27):** `model: 'xx' | 'heisenberg'` on the same decoherence pipeline. Falsification **AI** — XX n=8 and Heisenberg n=6 (tuned couple step / window) both pass X criteria.

## API

```typescript
import { runQeccProbeAsync } from '@/sim/runner-async'

const result = await runQeccProbeAsync({
  n: 8,
  field: 1.2,
  dt: 0.2,
  steps: 22,
  coupleStep: 7,
  coupling: 0.9,
  windowRadius: 2,
  model: 'xx', // or 'heisenberg' with n=6, field=2, coupleStep=5, coupling=0.85, windowRadius=3
})
// result.qecc.codeLabel, .fidelitySelectivity, .codeSubspaceFound, ...
```

WASM: `run_qecc_json` in `lib.rs` → `run_matter.rs::run_qecc_probe`.

## Honest limits

- Stabilizer search includes weight-1, weight-2, and weight-3 (ZZZ/XXX) same-letter Pauli generators (shipped 2026-06-26).
- “Code-like” and fidelity selectivity are consistent signatures, not a derivation that TFIM branches
  are a literal quantum error-correcting code.
- Mixed-state fidelity for weight-2 operators uses a product approximation (see `stabilizer_search.rs`).
