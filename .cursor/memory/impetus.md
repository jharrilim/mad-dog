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

## 2026-06-21 — Phase 1 S1 (kill circularity)

- **Model zoo:** `xx_chain`, `heisenberg_chain`, `sparse_local_chain` + shuffled factorization kinds.
- **Ensemble K:** `run_factorization_ensemble` — TFIM/XX/Heisenberg/sparse, ≥80% recovery.
- **Uniqueness report** on top-k equivalence classes (`line_equiv_distance` clustering).
- **Negative control M:** random nonlocal + `spectrumScramble` must not recover.
- **Deferred:** true eigenvalue-only blind inference; 3D/torus graph factorization.

## 2026-06-21 — Phase 3 S3 (relational time)

- **Multi-clock on grid/cube** — `kind` in `run_multi_clock`; falsification **G′**.
- **Modular vs uniform Δt** — edge Z + in-cone modular desync; **D′**.
- **Simultaneity surfaces** — `run_simultaneity_json`; foliation bend **N**.
- **`bench:time`** — grid/cube clocks + modular + simultaneity smoke.

## 2026-06-21 — Phase 4 S4→S9 (causality → Lorentz)

- **Lorentz scaling** — `run_lorentz_scaling_json`; cardinal CoV 3×3→4×4 (**L′**).
- **Dispersion** — wavepacket ω(k) proxy on TFIM chain (**O**).
- **Boost invariance** — `signalReferenceSite` observer; relΔ (**P**).
- **Scattering phase** — detrended exchange phase residual (**Q**).
- **`bench:causality`** — scaling + dispersion + boost + phase smoke.

## 2026-06-21 — Phase 2 S2 (space isn’t an artifact)

- **Lattice kinds** in `geometry_stability.rs`: chain, grid, cube quenches.
- **Cross-embedding ρ** — MDS vs Laplacian spectral layout on MI distances.
- **`dimStable`** — emergent dim does not collapse on quench (cube 2×2×3).
- **Falsification J′** — cube gauge-free geometry + dim stability.
- **`bench:geometry`** — dim vs manifold sweep (`run_geometry_dim_sweep_json`).

## 2026-06-21 — Phase 5 S5 (holography under dynamics)

- **RT quench time series** — per-step slope deficit + mean |S/cut − 1|; Spearman vs area pressure (**R′**).
- **Predictive refinement** — early warning when RT fit degrades before `needsRefinement`; split at warning step (**F′**).
- **Geodesic deviation proxy** — triangle defect in MI distance; consistency via geo–density or cross-proxy ρ (**C′**).
- **`bench:holography`** — RT quench + predictive + curvature proxy smoke.

## 2026-06-21 — Phase 6 S6–S8 (matter and classicality)

- **Stabilizer search** — Pauli sharpened on branches vs mixed; falsification **I′**.
- **Particle stability** — localization fraction h=0.5 vs 2.5; **S′**.
- **Branch Born** — env entropy vs overlap correlation; **T′**.
- **EFT DOF** — measured rank/site vs stabilizer code rate; **U′**.
- **`bench:matter`** — all four probes smoke-tested.

## 2026-06-21 — Phase 7 S10 (factor count dynamics)

- **In-place tensor split** — `embed_state_at_split` + evolve remainder; pressure relief vs pre-split (**V′**).
- **Holographic bound** — sweep n for area law + dim + Hamiltonian locality; `n_min` + saturation (**W′**).
- **`bench:factor`** — in-place split + bound smoke.

## 2026-06-21 — Phase 8 observational bridge

- **Lorentz headroom** — ε := speedCv vs GRB/synchrotron caps (`observational-lorentz.ts`).
- **RT calibration** — slope vs AdS/CFT anchor + quench breakdown export.
- **Time export** — multi-clock desync JSON for frozen-formalism metaphor.
- **IR noise floors** — code rate vs surface-code / gate fidelity order-of-magnitude.
- **`bench:observational`** — orchestrator → `exports/observational/*.json`.

## 2026-06-22 — Phase 9 planned (circularity hardening)

- **W′** — **Done** — blind factorization on |ψ⟩; hold-out ground seeds.
- **F′/V′/R′/C′** — **Done** — `refinement_holdout.rs` hold-out grid; battery pass on 3/3; relabeled prototype diagnostic.
- **U′** — **Done** — participation ratio (1/Σλ²) on branch window spectrum as third DOF estimate; two-of-three pass (rel=0.5, abs=0.35); `eft_dof.rs`.
- **T′/I′** — hold-out coupling/window bands.
- **`docs/circularity-audit.md`** — per-test independence index (F′ family section started).
