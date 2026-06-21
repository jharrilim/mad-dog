# Impetus — why we decided things

Dated bullets for decisions future agents should not relitigate without cause.

## 2025 — Project foundation

- **Correctness-first TS engine** over WebGPU first: need auditable physics before GPU speed; `runner.ts` as swap boundary.
- **Pauli-sum TFIM models** (chain / grid / cube): local Hamiltonians where geometry *should* emerge vs `randomNonlocal` as negative control.
- **`sim-check.ts` as regression oracle**: numerical claims (emergent dim, light cone, RT slope) must pass against WASM before we trust UI demos.

## Emergent space

- **MI → distance → classical MDS** (Cao–Carroll–Michalakis pipeline): matches paper essay, no black-box embedding.
- **Gap-based emergent dimension**: largest eigenvalue ratio drop on MDS Gram spectrum — preserves degenerate 2D planes.
- **h ≈ 1.5 for ground-state geometry demos**: paramagnetic phase gives decaying MI; ordered phase (low h) gives long-range correlations that muddy MDS.

## Emergent time

- **Reference-subtracted signal** `|⟨Z⟩_defect − ⟨Z⟩_ref|`: transverse field causes uniform ⟨Z⟩ precession on all sites; raw ⟨Z⟩ hides the light cone.
- **Product-state quench** (not ground state) for spacetime: need propagating disturbance, not equilibrium.
- **Adaptive Taylor substepping**: fixed large Δt caused energy drift; substep on `‖H‖·Δt ≲ 0.2`.

## Holography & relational time

- **Baby RT test** approved as honest speculation: area law vs Page curve + fit `S_A` vs boundary MI cut.
- **"Mass" = local X-flip + brief evolution**: concentrates entanglement; RT slope rose ~0.94 → ~1.4 in sim-check.
- **Relational clocks**: physical clock uses site signal thresholds; time map uses **tick indices** not Schrödinger t, or clocks looked perfectly synced falsely.

## 3+1 Universe Lab

- **Separate `/lab` page** with lazy-loaded Three.js: essay stays lean; browser routing with `404.html` SPA fallback on GitHub Pages.
- **2×2×2 cube default**: smallest 3D lattice that runs interactively; dim detector noisy at n=8 (reported honestly).
- **Procrustes3D alignment**: MDS gauge freedom would spin the 3D view between slices without it.
- **Fixed canvas height + block wheel propagation**: `h-full` in unconstrained grid caused infinite scroll; OrbitControls zoom scrolled the page.

## Parameters

- **Lower h → sharper quench worldlines; higher h → muddier tracks** (user observation): ordered vs paramagnetic dynamics — document in `docs/parameters-and-phases.md`, not a single h for all demos.

## 2026 — Rust WASM backend (GitHub Pages)

- **Single-threaded Rust/WASM in a Web Worker** — no `SharedArrayBuffer` / COOP-COEP; works on GitHub Pages.
- **WASM-only calculations (v0.7.0)** — TypeScript oracle retired; `src/sim/types.ts` + `runner-async.ts` wire JSON only; scripts validate Rust golden assertions.
- **Scattering fast path** — geometry-free light-cone evolution (`include_geometry: false`); sweeps use `lite: true` + `scripts/scattering-sweep.ts` with worker-thread parallelism.

## Documentation

- **`docs/` for research notes**, **`.cursor/memory/` for goals/non-goals/impetus** — different audiences and update triggers.
