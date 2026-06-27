# Phase 12 scaling research (2026-06-27)

Empirical probe of scaling, 2D spectrum-blind recovery, and uniqueness — prior to implementing the Phase 12 backlog in [roadmap.md](./roadmap.md).

**Artifacts:** `scripts/scaling-probe.ts` → `exports/scaling-probe.json`; Rust blind probe `cargo test scaling_probe_blind_spectrum -- --nocapture` in `locality_spectrum.rs`.

---

## Executive summary

| Area | Holds at scale? | Notes |
|------|-----------------|-------|
| Factorization spectrum (chain, semi-blind†) | **Yes** n=4–8 | 100% recovery, exact search to n=8 (~3 s); score gaps shrink ~32% n=6→8 |
| AA blind (MI+bandwidth, no Ĥ) chain | **Yes** n=4–8 | Lower scores (~0.72–0.73) than semi-blind; AA battery 9/9 at n=6 |
| AA blind 2D (grid/torus) | **No** | 3×3 grid and 2×2 torus both fail — **main new negative** |
| Uniqueness (top-k classes) | **Partially** | True labeling always in top-k; `bestClassSize=2` (reflection); gap narrows with n |
| Lorentz L′ (3×3 vs 4×4) | **Yes** | Speed CoV = 0.000 on both grids |
| Multi-clock G (chain) | **Breaks ~n≥11** | n=7,9 pass; n=11 fails (minPairwiseR²=0.977 > 0.95) |

† *Semi-blind* = `inputMode: spectrum` with default scorer (40% Ĥ locality + 30% MI + 25% bandwidth). Distinct from **AA-blind** (`spectrum_scrambled=true`, h_ref=None).

**What breaks:** relational-time signature on long chains; MI+bandwidth-only blind inference on 2D lattices.

**What holds:** TFIM chain factorization (Pauli and spectrum modes), AA-blind chain to n=8, Lorentz isotropy proxy, uniqueness with reflection degeneracy only.

---

## Measurements

### Factorization — shuffled TFIM chain (h=1.5, seed=4242, exact)

| n | Mode | Recovered | Best score | Score gap (2nd class) | Classes / best size | Time |
|---|------|-----------|------------|----------------------|---------------------|------|
| 4 | spectrum | ✓ | 0.923 | 0.194 | 3 / 2 | 4 ms |
| 5 | spectrum | ✓ | 0.919 | 0.140 | 3 / 2 | 6 ms |
| 6 | spectrum | ✓ | 0.917 | 0.108 | 3 / 2 | 18 ms |
| 7 | spectrum | ✓ | 0.914 | 0.088 | 3 / 2 | 144 ms |
| 8 | spectrum | ✓ | 0.911 | 0.074 | 3 / 2 | 3.0 s |
| 4–8 | pauli | ✓ | 0.946–0.970 | 0.092–0.251 | 3 / 2 | <15 ms |

n=8 uniqueness (spectrum, seed=100): same pattern — recovered ✓, gap=0.074, classes=3, bestClassSize=2.

### AA blind — MI+bandwidth only (`locality_spectrum`, h_ref=None)

| System | n | Recovered | Score | MI-NN ratio |
|--------|---|-----------|-------|-------------|
| chain | 4 | ✓ | 0.726 | 2.93 |
| chain | 5 | ✓ | 0.729 | 3.57 |
| chain | 6 | ✓ | 0.729 | 4.22 |
| chain | 7 | ✓ | 0.726 | 4.86 |
| chain | 8 | ✓ | 0.723 | 5.50 |
| **grid 3×3** | 9 | **✗** | 0.584 | 1.47 |
| **torus 2×2** | 4 | **✗** | 0.576 | 1.13 |

AA battery (n=6 only, 9 TFIM cases): **9/9** recovered (100%, pass threshold 67%).

### 2D spectrum search via WASM (`run_factorization_search_json`)

Semi-blind spectrum (uses Ĥ locality term; not AA-blind):

| Kind | Recovered | Method | permDist | Notes |
|------|-----------|--------|----------|-------|
| shuffled_grid 3×3 | ✗ | annealing | 6 | n=9 > exact limit; 18 s |
| shuffled_torus 2×2 | ✓ | exact | 0 | Pauli mode already passes (AH) |

**Config gap:** JSON `spectrumScramble` sets both `spectrum_scrambled` (blind) *and* shuffles eigenvector components (M negative control). True AA-blind 2D requires Rust path (`blind_lattice_case`) until a separate `blindSpectrum` flag is wired.

### Lorentz scaling (L′)

| Grid | Speed CoV | Pass |
|------|-----------|------|
| 3×3 | 0.000 | ✓ |
| 4×4 | 0.000 | ✓ |

`covImproves=true` (both zero). Dispersion O: slope=0.137, R²=0.997.

### Multi-clock chain (G criterion: defectUniformR²>0.95, minPairwiseR²<0.95)

| n | defectUniformR² | minPairwiseR² | inconsistentPairs | Pass |
|---|-----------------|---------------|-------------------|------|
| 7 | 1.000 | 0.112 | 4 | ✓ |
| 9 | 1.000 | 0.757 | 4 | ✓ |
| 11 | 1.000 | **0.977** | 0 | **✗** |

Falsification **G** uses n=9 (still passes). Falsification **AM** documents the n≥11 breakdown; `check:scaling` probes n=7/9/11.

---

## Literature notes

**Locality from the spectrum.** Cotler, Penington, and Ranard (*Comm. Math. Phys.* 368, 1267, 2019; [arXiv:1702.06142](https://arxiv.org/abs/1702.06142)) argue that for generic local Hamiltonians the energy spectrum *often* encodes a unique k-local tensor product structure — the optimistic pole of “locality from spectrum.” Our **AK** falsification is the minimal counterweight at the data level: with `{Eₙ}` alone, every permutation scores identically (permutation invariance), so labeling cannot be recovered. **AA** shows that adding low-energy **eigenvector** structure (MI + bandwidth on ψₙ) suffices for TFIM **chains** — consistent with needing more than eigenvalues but less than full Ĥ.

**Non-uniqueness and Hilbert-space fundamentalism.** Stoica ([arXiv:2103.15104](https://arxiv.org/abs/2103.15104), [IOP 1742-6596/2533/012027](https://doi.org/10.1088/1742-6596/2533/1/012027)) and related work argue that structures like preferred tensor factorizations cannot emerge *uniquely* from abstract quantum data alone if they are to be physically relevant — infinitely many equivalent TPSs can be constructed. Our sim operationalizes a narrower claim: given eigenvectors, search for a labeling that maximizes MI-nearest-neighbor ratio and spatial bandwidth. Uniqueness metrics (`equivalenceClassCount`, `scoreGapToSecondClass`) measure how sharp that optimum is; reflection gives a known 2-fold degeneracy on open chains.

**MI-based emergent geometry.** Carroll–Cao–Michalakis (*Phys. Rev. D* 95, 024031, 2017) and the Mad-Dog program use mutual information to define distances and MDS dimension — the same geometry pipeline as signatures **B/J**. You *Qi* et al. (*Phys. Rev. B* 97, 045153, 2018) learn spatial geometry from entanglement features on tensor networks. Our **2D blind failure** fits the pattern: MI on a small 2D TFIM ground state does not sharply distinguish grid/torus nearest-neighbor pairs from other pairings once qubit labels are shuffled — 1D chain topology is special (path graph has stronger NN vs non-NN MI contrast).

---

## Recommendations

### 1. Implement scaling battery first (Priority 1)

Automate pass-rate vs `n` for: factorization (exact to n=8, annealing beyond with luck reporting), **G** multi-clock chain sweep, AA blind chain extension, L′/L/O spot checks. Reuse `scripts/scaling-probe.ts` as seed; wire into falsification or `npm run bench:scaling`.

**Why first:** Surfaces *where* signatures degrade (G at n≈11) without committing to 2D scorer redesign; produces the Phase 12 “progress metric” from the roadmap.

**Status (2026-06-27):** `npm run check:scaling` runs the minimal battery; falsification **AL** encodes the 2D AA-blind negative; falsification **AM** encodes multi-clock G breaking at n≥11.

### 2. New falsification **AM** — implemented

**AM:** *Multi-clock G relational-time signature holds at n=9 but breaks at n≥11 (minPairwiseR² ≥ 0.95).*

Complements **G** (n=9 positive). Documents honest scaling boundary: long chains lose edge-clock desync while defect still tracks uniform Δt.

### 3. New falsification **AL** — implemented

**AL:** *AA-blind MI+bandwidth fails on 2D TFIM (3×3 grid, 2×2 torus) while chain n≤8 passes.*

Complements **AA** (chain positive) and **AK** ({Eₙ}-only impossible). Documents an honest boundary: spectrum-first locality is not yet a 2D signature.

### 4. 2D blind scorer (Priority 2) — tiebreak partial (2026-06-27)

**Shipped in `factorization.rs`:**

- **Graph-native far MI:** non-NN pairs use `site_graph_distance ≥ 2` (Manhattan / torus-wrap), not linear chain index `k+d`.
- **Lattice support bandwidth:** `support_bandwidth` uses graph diameter on active sites (grid Manhattan, torus wrap); line mode unchanged.
- **Exact-search tiebreak (2026-06-27):** when AA-blind on Grid/Torus, secondary sort uses `blind_lattice_tiebreak` = 10% MDS–grid Procrustes fit + 90% NN-edge MI heterogeneity (std-dev).

**Result:** AA-blind chain regression passes. 2D **still fails** exact recovery (`recovered=false` on 3×3 grid, 2×2 torus, seed 4242). Probe shows **true labeling scores below the top bucket** (true ≈0.535 vs max ≈0.584 on 3×3): spurious permutations inflate MI-NN ratio with artificially uniform edge weights. Tiebreak alone cannot fix recovery without rebalancing the primary 2D blind scorer.

**Still open:**

- Primary 2D blind score that penalizes uniform edge-MI patterns without breaking chain AA.
- Separate `blindSpectrum` JSON flag (decouple from eigenvector scrambling).
- Recovery metric: consider lattice symmetry class vs raw `perm_distance`.

Pauli-mode 2D search already passes (**AH**); the gap is specifically **blind** spectrum inference.

### 5. Uniqueness tightening — shipped (2026-06-27)

`npm run check:scaling` now asserts per-model uniqueness on TFIM n=4/6/8, XX n=6, sparse n=6: `trueInTopK`, `bestClassSize ≤ 2`, `equivalenceClassCount ≤ 3`, and `scoreGapToSecondClass` above n-dependent floors (sparse gap ≈0.005 at seed 4242). `bench:factorization` logs gap/classes for spectrum n=6/8 and XX n=6.

### 6. Negative controls at scale — shipped (2026-06-27)

Falsification **M** now runs random nonlocal + scrambled spectrum at **n=6 and n=8** (exact search); all four must fail recovery.

### 7. Blockers

| Blocker | Impact |
|---------|--------|
| n>8 exact search | 9! annealing for grid; report seed variance |
| `spectrumScramble` naming | Blocks WASM-only AA-blind 2D probes |
| G at n≥11 | S3 may need clock subset selection or longer dynamics — not a factorization bug |
| Recovery metric on 2D | `line_equiv_match` wrong for grid; use `perm_distance` |

---

## Related

- [factorization.md](./factorization.md), [falsification.md](./falsification.md) (AA, AK, AH, G, L′)
- [multi-clock.md](./multi-clock.md), [open-questions.md](./open-questions.md)
