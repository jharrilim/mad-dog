# Observational bridge (Phase 8)

Maps **toy simulator metrics** to **external literature caps** — export and documentation only, not new WASM physics. We do **not** claim the TFIM lattice units match GRB, synchrotron, or AdS/CFT numbers; we show **orders of magnitude** and **qualitative analogues**.

**Reproduce:** `npm run bench:observational` → JSON under `exports/observational/`.

See also: [roadmap.md](./roadmap.md) Phase 8, [falsification.md](./falsification.md) (internal thresholds vs observational caps).

---

## 1. Lorentz proxy ε(n) vs astrophysical caps

**Sim proxy:** `ε_eff := max(speedCv, boost relativeDelta, dispersion v_g CoV, |MI/lattice velocity ratio − 1|)`. Cardinal `speedCv` can underflow to 0 when grid fronts are degenerate; secondary proxies keep the export honest.

**Literature caps** (dimensionless, order of magnitude — see `scripts/observational-constants.ts`):

| Source | Typical bound | Role |
|--------|---------------|------|
| Fermi GBM GRB time-of-flight | δ_EE ~ 10⁻¹⁵ | Linear LIV in E/E_Planck |
| TeV GRB 221009A | ~10⁻¹⁷ | Stricter dispersion |
| Crab synchrotron electrons | ~10⁻⁸–10⁻¹⁰ | SME electron sector |

**Reading the export:** `headroom.ratio = ε_eff / bound`. Ratios ≫ 1 mean the **toy lattice anisotropy/mismatch exceeds** astrophysical LIV caps at accessible n (e.g. boost relΔ ~ 6% vs GRB δ ~ 10⁻¹⁵). `covImproves` tracks whether cardinal `speedCv` shrinks from 3×3 → 4×4.

---

## 2. Holographic RT vs AdS/CFT analogues

**Reference:** vacuum RT fit slope ≈ **1** (discrete Carroll–Singh / Cao–Carroll–Michalakis identity on ground states).

**Exports:**

| Regime | Sim fields | AdS/CFT analogue language |
|--------|------------|---------------------------|
| Ground / vacuum | `rtSlope`, `rtR2`, `deltaFromAdsCft` | Leading RT geodesic; slope deficit = discrete finite-n |
| Excitation density sweep | `densitySweep[]` | “Mass” deformation of RT slope (see [holography.md](./holography.md)) |
| Defect quench | `meanRtRatioDev`, `slopeDeviation`, `deviationDensityCorr` | Strong-subregion / excited-state RT breakdown |

Generic 1D area laws are **not** AdS/CFT-specific; we use CFT slope = 1 only as a **calibration anchor**.

---

## 3. Problem of time — multi-clock desync

**Metaphor:** Wheeler–DeWitt has no preferred time; Mad-Dog uses **relational** clocks. When `minPairwiseR2 < 0.95` while defect tracks uniform Δt (`defectUniformR2 > 0.95`), **no global simultaneity** exists across site clocks.

**Export highlights:**

- `grid` / `cube`: `minPairwiseR2`, `inconsistentPairs`
- `modular`: modular vs Z-edge vs uniform desync (**D′**)
- `simultaneity.slices[]`: `tauSkew` bend across emergent geometry (**N**)

Use `exports/observational/time.json` for frozen-formalism figures: each clock picks a different τ foliation.

---

## 4. IR subspace vs lab noise floors

**Sim:** Pauli stabilizer search + EFT DOF counting on branch windows ([qecc-probe.md](./qecc-probe.md), **I′** / **U′**).

**Proxy:** `redundancyGap := 1 − codeRate` (effective non-logical DOF per window).

**Literature floors** (order of magnitude):

| Source | ~floor | Comparison |
|--------|--------|------------|
| Surface-code per-cycle logical error | ~1% | `redundancyGap` vs 0.01 |
| Superconducting gate infidelity | ~0.1% | stabilizer sharpening margin |

Coupling sweep in export shows how `codeRate` and `measuredDofPerSite` move with env coupling — not a lab prediction, but a **noise-threshold vocabulary** for the proto-QECC diagnostic.

---

## Constants and scripts

| File | Purpose |
|------|---------|
| `scripts/observational-constants.ts` | Literature bounds + citations |
| `scripts/observational-lorentz.ts` | ε(n) + headroom |
| `scripts/observational-holography.ts` | RT calibration table |
| `scripts/observational-time.ts` | Clock desync export |
| `scripts/observational-ir.ts` | Code rate vs noise floors |
| `scripts/observational-bridge.ts` | Orchestrator |

---

## Honest limits

- Units are **not** matched to SI or Planck units; only dimensionless ratios are compared.
- Thresholds in [falsification.md](./falsification.md) are **hand-tuned on TFIM**; observational caps come from **external** literature.
- Passing the falsification battery does **not** imply compatibility with GRB or synchrotron bounds — check `headroom` explicitly.
