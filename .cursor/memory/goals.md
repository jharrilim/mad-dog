# Goals

- Explain [Carroll & Singh (2018)](https://arxiv.org/abs/1801.08132) Mad-Dog Everettianism with an honest, testable simulator — not toy cartoons.
- Detect and visualize **emergent space** (MI → MDS), **emergent time** (Page–Wootters slices), and **spacetime** (quench + light cones).
- Test speculations rigorously when we make them (holography area law, RT slope, relational clocks, 3+1 cube).
- Keep the reference quantum engine **correctness-first**, auditable TypeScript; UI calls `src/sim/runner.ts`.
- Ship as a GitHub Pages site: essay at `#/`, holistic **Universe Lab** at `#/lab`.
- Document learnings in `docs/`; keep `scripts/sim-check.ts` aligned with numerical claims.

## Near-term directions

- Worldline tracking, two-particle quenches, minimal decoherence branch.
- Code-split `/lab` to reduce essay page bundle weight.
- WebGPU/WASM backend validated against TS reference (when size demands it).
