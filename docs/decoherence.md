# Minimal decoherence / Everett branching

**Shipped (2026-06).** Live engine demo for branch-resolved worldlines after environment coupling.

## Model

| Piece | Implementation |
|-------|----------------|
| System | TFIM chain (`n` qubits), center defect quench |
| Environment | One extra qubit (index `n`) |
| Free phase | Chain evolves alone until `coupleStep` |
| Coupling | CNOT(center → env) at coupling step, then ZX terms between chain sites and `X_env` |

The environment records which-path information from the spreading defect. In Everett language, each env branch is a classical-looking track; the global state remains pure.

## API

```ts
import { runDecoherenceQuenchAsync } from '@/sim/runner-async'

const result = await runDecoherenceQuenchAsync({
  n: 8,
  field: 1.2,
  dt: 0.2,
  steps: 22,
  coupleStep: 7,
  coupling: 0.9,
  seed: 4242,
})
```

WASM: `run_decoherence_quench_json` in `decoherence.rs`.

## Outputs

- **Mixed worldline** — peak track on the chain marginal (dashed overlay; interference smear).
- **Branch worldlines** — conditional tracks given env = |0⟩ or |1⟩ (solid overlays).
- **Env entropy series** — rises after coupling when branches become distinguishable.
- **`sharpenRatio`** — max of spatial-entropy and worldline-jitter ratios (mixed vs branch-averaged) after coupling.

## UI

- **Experiments** → *Decoherence quench — branch-resolved worldlines* (`DecoherenceQuenchViz`)
- Essay page still has the static *Decoherence & Branching* stepped diagram (`DecoherenceViz` on Home).

## Honest limits

- One env qubit, hand-built coupling — not derived from a macroscopic bath.
- Branch tracks use **conditional amplitudes** (Everett relative states), not a collapse postulate.
- Does **not** identify QECC / EFT code subspaces — prerequisite for Carroll–Singh’s open infrared-matter question. See [open-questions.md](./open-questions.md) (*Effective field theory / QECC*).

## Checks

- `npm run check:wasm` — smoke test on default preset
- Rust unit test `decoherence_branches_sharpen`
