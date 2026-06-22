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

## 2026-06 — Adaptive holographic splitting

- **Split trigger** at first `needs_refinement` step; **accept rule** compares pressure at n vs n+Δ at same quench depth.
- **`suggest_split_site`** — entropy-gradient hint for where to add a factor (toy).
- **`npm run sweep:refinement`**, falsification test **H**, `AdaptiveRefinementViz` on Experiments.

## 2026-06 — Multi-clock consistency networks

- **N physical clocks** on one quench; pairwise R² matrix vs uniform Δt reference.
- **Falsification G** — defect syncs (R² > 0.95) but min pairwise R² < 0.95 on n=9 chain (no global time).
- **5-clock preset** on Experiments page spreads sites across chain for richer desync structure.

- **Single-defect peak track** on spacetime / Universe Lab (`track_single_worldline`); overlays on heatmaps and 3D MDS path.
- **Scattering metrics** — separation series, velocity fits, `bothMoved`; separation chart + h slider in UI.
- **Exploration backlog** in `docs/roadmap.md` so follow-ups (multi-clock, decoherence, dynamical refinement) are not lost.

## 2026-06 — Minimal decoherence quench

- **One env qubit + ZX coupling + CNOT at couple step** — ZZ alone cannot flip env from |0⟩; needed X_env terms for branching.
- **Branch tracks = conditional chain states** (Everett relative states), not collapse; mixed dashed worldline kept for comparison.
- **QECC / EFT subspace ID** explicitly still deferred — decoherence is prerequisite, not substitute.

## 2026-06 — 3D dim resolution (2×2×3)

- **Gap-only scree failed** on 8-point cube ground (degenerate λ) and overstated dim on 12-point ground.
- **Kaiser fallback** (λ > mean) when first gap is weak — 2×2×3 ground + quench k=3 report dim=3; 2×2×2 ground stays inconclusive at dim=1.
- **`bench:universe`** + wasm-check assertions on conclusive cases only.

## 2026-06 — Mass / RT slope vs excitation density

- **Multi-insertion** (`inject_mass_multi`) at fixed strength; Δslope vs ρ grows ~linearly on n=10 TFIM.
- **`sweep:mass`** CLI + density chart on RtMassViz; single-site deformation was already in UI.

## Documentation

- **`docs/` for research notes**, **`.cursor/memory/` for goals/non-goals/impetus** — different audiences and update triggers.
