//! Falsification battery — Mad-Dog-specific claim tests (WASM-only).

use crate::geometry::{entropy_of_region, mutual_information_matrix};
use crate::models::{line_locality_fraction, tfim_chain, tfim_torus};
use crate::quantum::ground_state;
use crate::rng::Rng;
use crate::run_factorization::{run_factorization_search, FactorizationSearchConfig};
use crate::run_factorization_ensemble::run_factorization_ensemble;
use crate::run_light_cone_compare::{run_light_cone_compare, LightConeCompareConfig};
use crate::modular_time::ModularDualClockConfig;
use crate::run_modular_dual_clock::run_modular_dual_clock;
use crate::run_modular_multi_clock::{run_modular_multi_clock, ModularMultiClockRunConfig};
use crate::eigenvalue_only::run_eigenvalue_only_battery;
use crate::run_refinement::{
    run_adaptive_refinement, run_predictive_refinement, run_refinement_quench,
    AdaptiveRefinementConfig, RefinementQuenchConfig,
};
use crate::run_matter::{
    run_branch_born_probe, run_eft_dimension_probe, run_particle_stability_probe,
    run_qecc_probe, run_stabilizer_search, BranchBornConfig, EftDimensionConfig,
    ParticleStabilityConfig, StabilizerSearchConfig,
};
use crate::run_factor_dynamics::{
    run_holographic_bound_probe, run_inplace_split_probe,
};
use crate::holographic_bound::HolographicBoundConfig;
use crate::tensor_split::InplaceSplitConfig;
use crate::matter_holdout::{
    eval_i_prime_holdout, eval_t_prime_holdout,
    BornHoldoutCase, BORN_TUNING_DEFAULT,
    StabHoldoutCase, STAB_TUNING_DEFAULT,
};
use crate::refinement_holdout::{
    eval_c_prime_holdout, eval_f_prime_holdout, eval_r_prime_holdout, eval_v_prime_holdout,
    REFINEMENT_TUNING_DEFAULT,
};
use crate::run_rt_quench::{run_curvature_proxy_quench, run_rt_quench, CurvatureProxyQuenchConfig, RtQuenchConfig};
use crate::excitation_subspace::ExcitationSubspaceConfig;
use crate::run_excitation_subspace::run_excitation_subspace_probe;
use crate::run_geometry_stability::{run_geometry_stability, GeometryStabilityConfig};
use crate::run_multi_clock::{run_multi_clock, MultiClockRunConfig};
use crate::boost_invariance::{run_boost_invariance, BoostInvarianceConfig};
use crate::poincare::{run_poincare_composite, PoincareCompositeConfig};
use crate::dispersion::{run_dispersion, DispersionConfig};
use crate::lorentz::grid_cardinal_speed_cv;
use crate::run_lorentz_scaling::run_lorentz_scaling;
use crate::run_scaling_battery::run_multi_clock_g_scaling_boundary;
use crate::scattering::{run_two_defect_scattering, lattice_separation, ScatteringConfig};
use crate::locality_spectrum::{run_aa_blind_2d_boundary, run_locality_spectrum_battery};
use serde::Serialize;

fn rt_ratio_std(n: usize, field: f64, seed: u32) -> f64 {
    let model = tfim_chain(n, 1.0, field);
    let (state, _, _) = ground_state(&model.hamiltonian, &mut Rng::new(seed), 4000, 1e-9);
    let mi = mutual_information_matrix(&state);
    let mut ratios = Vec::new();
    for start in 0..n {
        for len in 1..=(n - start) {
            let region: Vec<usize> = (start..start + len).collect();
            let s_a = entropy_of_region(&state, &region);
            let end = start + len - 1;
            let mut boundary_mi = 0.0;
            for a in start..=end {
                for b in 0..n {
                    if b < start || b > end {
                        boundary_mi += mi[a][b];
                    }
                }
            }
            if boundary_mi > 1e-9 {
                ratios.push(s_a / (0.5 * boundary_mi));
            }
        }
    }
    if ratios.len() < 2 {
        return 0.0;
    }
    let mean = ratios.iter().sum::<f64>() / ratios.len() as f64;
    let var = ratios.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / ratios.len() as f64;
    var.sqrt()
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FalsificationTest {
    pub id: String,
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FalsificationBatteryResult {
    pub tests: Vec<FalsificationTest>,
    pub passed: usize,
    pub total: usize,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

pub fn run_falsification_battery() -> FalsificationBatteryResult {
    let mut tests = Vec::new();

    // A — blind locality recovery
    {
        let r = run_factorization_search(&FactorizationSearchConfig {
            kind: "shuffled_chain".to_string(),
            n: 8,
            field: 1.5,
            seed: 100,
            top_k: 5,
            input_mode: None,
            search_method: Some("exact".to_string()),
            eigenstate_count: None,
            graph_kind: None,
            rows: None,
            cols: None,
            lx: None,
            ly: None,
            lz: None,
            distance_decay: None,
            annealing_steps: None,
            spectrum_scramble: None,
        });
        tests.push(FalsificationTest {
            id: "A".to_string(),
            name: "Blind locality recovery (shuffled chain n=8)".to_string(),
            passed: r.recovered_identity,
            detail: format!(
                "recovered={}, best score={:.3}",
                r.recovered_identity, r.best.score
            ),
        });
    }

    // A' — cross-graph factorization signal
    {
        let grid_on_grid = run_factorization_search(&FactorizationSearchConfig {
            kind: "shuffled_grid".to_string(),
            n: 9,
            field: 1.5,
            seed: 4242,
            top_k: 5,
            input_mode: None,
            search_method: Some("exact".to_string()),
            eigenstate_count: None,
            graph_kind: Some("grid".to_string()),
            rows: Some(3),
            cols: Some(3),
            lx: None,
            ly: None,
            lz: None,
            distance_decay: None,
            annealing_steps: None,
            spectrum_scramble: None,
        });
        let grid_on_line = run_factorization_search(&FactorizationSearchConfig {
            kind: "shuffled_grid".to_string(),
            n: 9,
            field: 1.5,
            seed: 4242,
            top_k: 5,
            input_mode: None,
            search_method: Some("exact".to_string()),
            eigenstate_count: None,
            graph_kind: Some("line".to_string()),
            rows: Some(3),
            cols: Some(3),
            lx: None,
            ly: None,
            lz: None,
            distance_decay: None,
            annealing_steps: None,
            spectrum_scramble: None,
        });
        let score_gap = grid_on_grid.best.score - grid_on_line.best.score;
        let torus_locality = line_locality_fraction(&tfim_torus(3, 3, 1.0, 1.5).hamiltonian);
        tests.push(FalsificationTest {
            id: "A'".to_string(),
            name: "Cross-graph factorization signal".to_string(),
            passed: score_gap < 0.05 && torus_locality < 0.9,
            detail: format!(
                "grid-line gap={:.3}, torus line locality={:.0}%",
                score_gap,
                torus_locality * 100.0
            ),
        });
    }

    // B — MI geometry vs lattice LR velocity
    {
        let lc = run_light_cone_compare(&LightConeCompareConfig {
            n: 10,
            field: 1.5,
            dt: 0.2,
            steps: 20,
            seed: 42,
        });
        let ratio = lc.comparison.velocity_ratio;
        tests.push(FalsificationTest {
            id: "B".to_string(),
            name: "MI emergent distance vs lattice LR velocity".to_string(),
            passed: ratio > 0.0 && (ratio - 1.0).abs() < 0.25,
            detail: format!("v_mi/v_lattice={:.3}", ratio),
        });
    }

    // C — RT ratio stability on ground state
    {
        let std = rt_ratio_std(10, 1.5, 42);
        tests.push(FalsificationTest {
            id: "C".to_string(),
            name: "RT ratio S_A / (½ boundary MI) stable on ground".to_string(),
            passed: std < 0.15,
            detail: format!("std={:.3} across intervals", std),
        });
    }

    // D — modular clocks desync in-cone
    {
        let m = run_modular_dual_clock(&ModularDualClockConfig {
            n: 12,
            field: 1.2,
            dt: 0.15,
            steps: 48,
            modular_slices: 12,
        });
        tests.push(FalsificationTest {
            id: "D".to_string(),
            name: "Modular in-cone clocks desynchronize".to_string(),
            passed: m.sync_r2_modular < 0.95,
            detail: format!(
                "syncR2_modular={:.3}, syncR2_Z={:.3}",
                m.sync_r2_modular, m.sync_r2_z
            ),
        });
    }

    // D′ — modular and Z clocks both desync from uniform Δt
    {
        let m = run_modular_dual_clock(&ModularDualClockConfig {
            n: 12,
            field: 1.2,
            dt: 0.15,
            steps: 48,
            modular_slices: 12,
        });
        tests.push(FalsificationTest {
            id: "D′".to_string(),
            name: "Modular and Z clocks desync from uniform Δt".to_string(),
            passed: m.min_modular_uniform_r2 < 0.95 && m.min_z_edge_uniform_r2 < 0.95,
            detail: format!(
                "minModUniformR2={:.3}, minZEdgeUniformR2={:.3}",
                m.min_modular_uniform_r2, m.min_z_edge_uniform_r2
            ),
        });
    }

    // E — two excitations propagate without binding
    {
        let s = run_two_defect_scattering(&ScatteringConfig {
            defect_sites: Some([3, 8]),
            lite: true,
            taylor_order: 4,
            ..Default::default()
        });
        let d1 = s.defect_sites[0];
        let d2 = s.defect_sites[1];
        let left_moved = s.worldlines[0]
            .iter()
            .any(|p| p.site.abs_diff(d1) > 1);
        let right_moved = s.worldlines[1]
            .iter()
            .any(|p| p.site.abs_diff(d2) > 1);
        tests.push(FalsificationTest {
            id: "E".to_string(),
            name: "Two excitations propagate without binding".to_string(),
            passed: left_moved && right_moved && s.min_separation >= 1.0,
            detail: format!(
                "crossed={}, minSep={:.1}, moved=({}, {})",
                s.crossed, s.min_separation, left_moved, right_moved
            ),
        });
    }

    // F — refinement diagnostic decoupling
    {
        let q = run_refinement_quench(&RefinementQuenchConfig {
            n: 10,
            field: 1.5,
            dt: 0.2,
            steps: 16,
            seed: 7711,
        });
        let lag = q.decoupling_lag;
        let late = q.slices.last().map(|s| &s.diagnostics);
        let slope_deficit = late.map(|d| (d.rt_slope - 1.0).abs() / 0.35).unwrap_or(0.0);
        tests.push(FalsificationTest {
            id: "F".to_string(),
            name: "Refinement components decouple under quench".to_string(),
            passed: lag > 0 || slope_deficit > 0.1,
            detail: format!(
                "peak lag={lag} steps, late slopeDeficit={slope_deficit:.3}"
            ),
        });
    }

    // G — multi-clock network: defect syncs with uniform, network not globally consistent
    {
        let n = 9;
        let m = run_multi_clock(&MultiClockRunConfig {
            kind: "chain".to_string(),
            n: Some(n),
            rows: None,
            cols: None,
            lz: None,
            field: 1.0,
            dt: 0.2,
            steps: 40,
            clock_sites: Some(vec![0, n / 2, n - 1]),
            physical_slices: Some(15),
        });
        tests.push(FalsificationTest {
            id: "G".to_string(),
            name: "Multi-clock network lacks global consistency".to_string(),
            passed: m.defect_uniform_r2 > 0.95 && m.min_pairwise_r2 < 0.95,
            detail: format!(
                "defectUniformR2={:.3}, minPairwiseR2={:.3}, edgeEdgeR2={:.3}, inconsistentPairs={}",
                m.defect_uniform_r2, m.min_pairwise_r2, m.edge_edge_r2, m.inconsistent_pairs
            ),
        });
    }

    // G′ — multi-clock on 3×3 grid: defect local, network not globally consistent
    {
        let m = run_multi_clock(&MultiClockRunConfig {
            kind: "grid".to_string(),
            n: None,
            rows: Some(3),
            cols: Some(3),
            lz: None,
            field: 1.0,
            dt: 0.2,
            steps: 40,
            clock_sites: None,
            physical_slices: Some(15),
        });
        tests.push(FalsificationTest {
            id: "G′".to_string(),
            name: "Grid multi-clock network lacks global consistency".to_string(),
            passed: m.defect_uniform_r2 > 0.95 && m.min_pairwise_r2 < 0.95,
            detail: format!(
                "{} defectUniformR2={:.3}, minPairwiseR2={:.3}, edgeEdgeR2={:.3}",
                m.label, m.defect_uniform_r2, m.min_pairwise_r2, m.edge_edge_r2
            ),
        });
    }

    // N — emergent simultaneity surfaces bend between observers
    {
        use crate::run_simultaneity::{run_simultaneity, SimultaneityRunConfig};
        let n = 9;
        let s = run_simultaneity(&SimultaneityRunConfig {
            n,
            field: 1.0,
            dt: 0.2,
            steps: 40,
            clock_a: Some(n / 2),
            clock_b: Some(0),
            physical_slices: Some(15),
            embed_dim: Some(2),
            reference_site: Some(0),
        });
        tests.push(FalsificationTest {
            id: "N".to_string(),
            name: "Emergent simultaneity surfaces bend between observers".to_string(),
            passed: s.bend_detected,
            detail: format!(
                "meanTauSkew={:.3}, slopeDelta={:.3}, maxSkew={:.1}",
                s.mean_tau_skew, s.slope_delta, s.max_tau_skew
            ),
        });
    }

    // H — adaptive split accepted when triggered
    {
        let a = run_adaptive_refinement(&AdaptiveRefinementConfig {
            n: 10,
            field: 1.5,
            dt: 0.2,
            steps: 18,
            seed: 7711,
            delta_n: Some(2),
        });
        let (passed, detail) = match &a.split_event {
            Some(ev) => (
                ev.accepted,
                format!(
                    "triggerStep={}, accepted={}, deltaP={:.3}",
                    ev.trigger_step, ev.accepted, ev.pressure_delta
                ),
            ),
            None => (false, "no split trigger".to_string()),
        };
        tests.push(FalsificationTest {
            id: "H".to_string(),
            name: "Adaptive split relieves pressure when triggered".to_string(),
            passed,
            detail,
        });
    }

    // I — branch-resolved excitations look code-like after decoherence
    {
        let q = run_excitation_subspace_probe(&ExcitationSubspaceConfig {
            n: 8,
            field: 1.2,
            dt: 0.2,
            steps: 22,
            couple_step: 7,
            coupling: 0.9,
            seed: 4242,
            window_radius: 2,
            model: None,
        });
        tests.push(FalsificationTest {
            id: "I".to_string(),
            name: "Branch excitations sharpen into low-rank Pauli-biased subspace".to_string(),
            passed: q.code_like,
            detail: format!(
                "gain={:.3}, rankRed={:.3}, overlap={:.3}, mixedSharp={:.3}",
                q.sharpness_gain, q.rank_reduction, q.branch_overlap, q.mixed_sharpness
            ),
        });
    }

    // J — MI distance ranking stable without Procrustes (ordered phase, 1D chain)
    {
        let ordered = run_geometry_stability(&GeometryStabilityConfig {
            kind: "chain".to_string(),
            n: 10,
            rows: None,
            cols: None,
            lz: None,
            field: 1.2,
            dt: 0.2,
            steps: 20,
            xi: 1.0,
            seed: 42,
        });
        tests.push(FalsificationTest {
            id: "J".to_string(),
            name: "Gauge-free MI geometry stable in ordered phase (chain)".to_string(),
            passed: ordered.geometry_stable,
            detail: format!(
                "drift={:.3} corr={:.3} embed={:.3} dimStd={:.3}",
                ordered.mean_distance_drift,
                ordered.mean_rank_correlation,
                ordered.mean_embedding_correlation,
                ordered.dim_std
            ),
        });
    }

    // J' — 3D cube quench: gauge-free geometry stable without Procrustes
    {
        let cube = run_geometry_stability(&GeometryStabilityConfig {
            kind: "cube".to_string(),
            n: 12,
            rows: Some(2),
            cols: Some(2),
            lz: Some(3),
            field: 1.5,
            dt: 0.2,
            steps: 20,
            xi: 1.0,
            seed: 42,
        });
        tests.push(FalsificationTest {
            id: "J'".to_string(),
            name: "Gauge-free MI geometry stable on 2×2×3 cube quench".to_string(),
            passed: cube.geometry_stable && cube.dim_stable,
            detail: format!(
                "corr={:.3} embed={:.3} dimMean={:.2} expected={} dimStable={}",
                cube.mean_rank_correlation,
                cube.mean_embedding_correlation,
                cube.mean_emergent_dim,
                cube.expected_dim,
                cube.dim_stable
            ),
        });
    }

    // K — ensemble spectrum-only blind recovery (signature S1; Phase 1 model zoo)
    {
        let ensemble = run_factorization_ensemble();
        let mut by_kind: std::collections::BTreeMap<String, (usize, usize)> =
            std::collections::BTreeMap::new();
        for c in &ensemble.cases {
            let entry = by_kind.entry(c.kind.clone()).or_insert((0, 0));
            entry.1 += 1;
            if c.recovered_identity {
                entry.0 += 1;
            }
        }
        let breakdown: Vec<String> = by_kind
            .iter()
            .map(|(k, (r, t))| format!("{k}={r}/{t}"))
            .collect();
        tests.push(FalsificationTest {
            id: "K".to_string(),
            name: "Ensemble spectrum-only blind locality recovery".to_string(),
            passed: ensemble.passed,
            detail: format!(
                "recovery {:.0}% ({}/{}) [{}]",
                ensemble.recovery_rate * 100.0,
                ensemble.recovered,
                ensemble.total,
                breakdown.join(", ")
            ),
        });
    }

    // M — negative controls (random nonlocal + scrambled spectrum), n=6 and n=8
    {
        let neg = |n: usize, seed: u32, kind: &str, scramble: bool| -> bool {
            run_factorization_search(&FactorizationSearchConfig {
                kind: kind.to_string(),
                n,
                field: 1.5,
                seed,
                top_k: 3,
                input_mode: Some("spectrum".to_string()),
                search_method: Some(if n <= 8 {
                    "exact".to_string()
                } else {
                    "annealing".to_string()
                }),
                eigenstate_count: Some(if n <= 4 { 2 } else { 3 }),
                graph_kind: None,
                rows: None,
                cols: None,
                lx: None,
                ly: None,
                lz: None,
                distance_decay: None,
                annealing_steps: None,
                spectrum_scramble: if scramble { Some(true) } else { None },
            })
            .recovered_identity
        };
        let random6 = neg(6, 777, "random", false);
        let scrambled6 = neg(6, 4242, "shuffled_chain", true);
        let random8 = neg(8, 888, "random", false);
        let scrambled8 = neg(8, 4242, "shuffled_chain", true);
        let passed = !random6 && !scrambled6 && !random8 && !scrambled8;
        tests.push(FalsificationTest {
            id: "M".to_string(),
            name: "Negative controls reject fake locality".to_string(),
            passed,
            detail: format!(
                "n6 random={random6} scrambled={scrambled6}; n8 random={random8} scrambled={scrambled8} (all should be false)"
            ),
        });
    }

    // L — emergent causal isotropy proxy (signature S9 v0)
    {
        const CV_MAX: f64 = 0.25;
        let cv = grid_cardinal_speed_cv(3, 3, 1.2, 0.2, 28);
        tests.push(FalsificationTest {
            id: "L".to_string(),
            name: "Directional front speeds isotropic on 3×3 grid (Lorentz proxy v0)".to_string(),
            passed: cv < CV_MAX,
            detail: format!("cardinal speed CoV={cv:.3} (pass if < {CV_MAX})"),
        });
    }

    // L′ — cardinal speed CoV improves with lattice size (scaling)
    {
        let scaling = run_lorentz_scaling();
        tests.push(FalsificationTest {
            id: "L′".to_string(),
            name: "Cardinal speed CoV improves with grid size".to_string(),
            passed: scaling.all_passed,
            detail: format!(
                "covSmall={:.3}, covLarge={:.3}, improves={}",
                scaling.cov_small, scaling.cov_large, scaling.cov_improves
            ),
        });
    }

    // O — dispersion linear at small k
    {
        let d = run_dispersion(&DispersionConfig {
            n: 16,
            field: 1.0,
            dt: 0.15,
            steps: 48,
            modes: 3,
        });
        tests.push(FalsificationTest {
            id: "O".to_string(),
            name: "Dispersion ω(k) linear at small k".to_string(),
            passed: d.linear_at_small_k,
            detail: format!(
                "slope={:.3}, intercept={:.3}, R²={:.3}",
                d.omega_slope, d.omega_intercept, d.linear_r2
            ),
        });
    }

    // P — weak boost invariance (clock-subset observers)
    {
        let b = run_boost_invariance(&BoostInvarianceConfig {
            rows: 4,
            cols: 4,
            field: 1.2,
            dt: 0.2,
            steps: 32,
            edge_site: Some(0),
        });
        tests.push(FalsificationTest {
            id: "P".to_string(),
            name: "Light-cone speed invariant under clock-subset observer".to_string(),
            passed: b.shape_invariant,
            detail: format!(
                "vUniform={:.3}, vEdgeClock={:.3}, relDelta={:.3}",
                b.velocity_uniform, b.velocity_edge_clock, b.relative_delta
            ),
        });
    }

    // Q — scattering exchange phase stabilizes post-interaction
    {
        let s = run_two_defect_scattering(&ScatteringConfig {
            defect_sites: Some([3, 8]),
            lite: true,
            taylor_order: 4,
            ..Default::default()
        });
        tests.push(FalsificationTest {
            id: "Q".to_string(),
            name: "Scattering exchange phase stable after interaction".to_string(),
            passed: s.phase_stable,
            detail: format!(
                "postPhaseStd={:.3}, minSep={:.1}",
                s.post_interaction_phase_std, s.min_separation
            ),
        });
    }

    // AD — exchange phase shifts measurably at cone overlap
    {
        let s = run_two_defect_scattering(&ScatteringConfig {
            defect_sites: Some([3, 8]),
            lite: true,
            taylor_order: 4,
            ..Default::default()
        });
        let passed = s.overlap_detected
            && s.interaction_phase_shift.abs() > 0.05
            && s.separation_time_delay.is_finite();
        tests.push(FalsificationTest {
            id: "AD".to_string(),
            name: "Exchange phase shifts measurably at cone overlap".to_string(),
            passed,
            detail: format!(
                "overlapStep={}, phaseShift={:.3}, timeDelay={:.3}, minSep={:.1}",
                s.overlap_step,
                s.interaction_phase_shift,
                s.separation_time_delay,
                s.min_separation
            ),
        });
    }

    // AE — two-defect scattering on 2D grid (Phase 11)
    {
        let s = run_two_defect_scattering(&ScatteringConfig {
            n: 9,
            kind: Some("grid".to_string()),
            rows: Some(3),
            cols: Some(3),
            defect_sites: Some([0, 8]),
            lite: true,
            taylor_order: 4,
            ..Default::default()
        });
        let init_sep = {
            let pos = &s.layout_positions;
            let a = s.defect_sites[0].min(pos.len().saturating_sub(1));
            let b = s.defect_sites[1].min(pos.len().saturating_sub(1));
            lattice_separation(a, b, pos)
        };
        let passed = s.kind == "grid" && s.both_moved && s.min_separation < init_sep;
        tests.push(FalsificationTest {
            id: "AE".to_string(),
            name: "Two-defect scattering on 2D TFIM grid".to_string(),
            passed,
            detail: format!(
                "bothMoved={}, minSep={:.1}, initSep={:.1}, label={}",
                s.both_moved, s.min_separation, init_sep, s.label
            ),
        });
    }

    // AF — effective mass from ω² = m² + v²k² on same chain as scattering
    {
        let s = run_two_defect_scattering(&ScatteringConfig {
            defect_sites: Some([3, 8]),
            lite: true,
            taylor_order: 4,
            ..Default::default()
        });
        let passed = s.kind == "chain"
            && s.effective_mass.is_finite()
            && s.effective_mass > 0.01
            && s.dispersion_velocity_mean > 0.01;
        tests.push(FalsificationTest {
            id: "AF".to_string(),
            name: "Effective mass from dispersion ω(k) on scattering chain".to_string(),
            passed,
            detail: format!(
                "m_eff={:.3}, v_g={:.3}, v_scatter/v_g={:.2}",
                s.effective_mass,
                s.dispersion_velocity_mean,
                s.velocity_dispersion_ratio
            ),
        });
    }

    // AG — same |ψ⟩ factorization stable under embed / truncate
    {
        use crate::factorization_compare::{run_factorization_compare, FactorizationCompareConfig};
        let r = run_factorization_compare(&FactorizationCompareConfig {
            n_small: 8,
            field: 1.5,
            dt: 0.2,
            step: 12,
            seed: 4242,
            delta_n: Some(2),
            split_site: None,
        });
        let passed = r.embedding_faithful
            && r.roundtrip_fidelity > 0.99
            && r.locality_drift_roundtrip < 0.05;
        tests.push(FalsificationTest {
            id: "AG".to_string(),
            name: "Same |ψ⟩ factorization stable under embed/truncate".to_string(),
            passed,
            detail: format!(
                "F={:.4}, Δloc_rt={:.3}, Δloc_emb={:.3}, Δdim_rt={}, faithful={}",
                r.roundtrip_fidelity,
                r.locality_drift_roundtrip,
                r.locality_drift_embed,
                r.dim_drift_roundtrip,
                r.embedding_faithful
            ),
        });
    }

    // AH — 3D cube and periodic torus factorization search targets
    {
        let cube = run_factorization_search(&FactorizationSearchConfig {
            kind: "shuffled_cube".to_string(),
            n: 8,
            field: 1.5,
            seed: 4242,
            top_k: 5,
            input_mode: Some("pauli".to_string()),
            search_method: Some("exact".to_string()),
            eigenstate_count: Some(1),
            graph_kind: Some("cube".to_string()),
            rows: None,
            cols: None,
            lx: Some(2),
            ly: Some(2),
            lz: Some(2),
            distance_decay: None,
            annealing_steps: None,
            spectrum_scramble: None,
        });
        let torus = run_factorization_search(&FactorizationSearchConfig {
            kind: "shuffled_torus".to_string(),
            n: 4,
            field: 1.5,
            seed: 4242,
            top_k: 5,
            input_mode: Some("pauli".to_string()),
            search_method: Some("exact".to_string()),
            eigenstate_count: Some(1),
            graph_kind: Some("torus".to_string()),
            rows: Some(2),
            cols: Some(2),
            lx: None,
            ly: None,
            lz: None,
            distance_decay: None,
            annealing_steps: None,
            spectrum_scramble: None,
        });
        let passed = cube.recovered_identity
            && torus.recovered_identity
            && cube.best.locality_fraction > 0.9
            && torus.best.locality_fraction > 0.9;
        tests.push(FalsificationTest {
            id: "AH".to_string(),
            name: "Cube/torus factorization search recovers shuffled TFIM".to_string(),
            passed,
            detail: format!(
                "cube loc={:.0}% recovered={}; torus loc={:.0}% recovered={}",
                cube.best.locality_fraction * 100.0,
                cube.recovered_identity,
                torus.best.locality_fraction * 100.0,
                torus.recovered_identity
            ),
        });
    }

    // AI — QECC code subspace on non-TFIM chains (XX + Heisenberg)
    {
        let xx = run_qecc_probe(&StabilizerSearchConfig {
            excitation: ExcitationSubspaceConfig {
                n: 8,
                field: 1.2,
                dt: 0.2,
                steps: 22,
                couple_step: 7,
                coupling: 0.9,
                seed: 4242,
                window_radius: 2,
                model: Some("xx".to_string()),
            },
        });
        let heisenberg = run_qecc_probe(&StabilizerSearchConfig {
            excitation: ExcitationSubspaceConfig {
                n: 6,
                field: 2.0,
                dt: 0.2,
                steps: 22,
                couple_step: 5,
                coupling: 0.85,
                seed: 4242,
                window_radius: 3,
                model: Some("heisenberg".to_string()),
            },
        });
        let passed = xx.qecc.code_subspace_found
            && xx.qecc.fidelity_selectivity > 1.1
            && heisenberg.qecc.code_subspace_found
            && heisenberg.qecc.fidelity_selectivity > 1.1;
        tests.push(FalsificationTest {
            id: "AI".to_string(),
            name: "QECC code subspace on non-TFIM chains (XX + Heisenberg)".to_string(),
            passed,
            detail: format!(
                "xx sel={:.2}× {}; heisenberg sel={:.2}× {}",
                xx.qecc.fidelity_selectivity,
                xx.excitation.model,
                heisenberg.qecc.fidelity_selectivity,
                heisenberg.excitation.model
            ),
        });
    }

    // AJ — modular-flow multi-clock on 3×3 grid: defect local, network not globally consistent
    {
        let m = run_modular_multi_clock(&ModularMultiClockRunConfig {
            kind: "grid".to_string(),
            n: None,
            rows: Some(3),
            cols: Some(3),
            lz: None,
            field: 1.0,
            dt: 0.2,
            steps: 40,
            clock_sites: None,
            modular_slices: Some(12),
        });
        tests.push(FalsificationTest {
            id: "AJ".to_string(),
            name: "Grid modular-flow clock network lacks global consistency".to_string(),
            passed: m.min_pairwise_r2 < 0.95 && m.inconsistent_pairs >= 1,
            detail: format!(
                "{} defectUniformR2={:.3}, minPairwiseR2={:.3}, edgeEdgeR2={:.3}",
                m.label, m.defect_uniform_r2, m.min_pairwise_r2, m.edge_edge_r2
            ),
        });
    }

    // AK — {Eₙ}-only cannot recover factorization; spectrum+ψ control still can
    {
        let r = run_eigenvalue_only_battery();
        tests.push(FalsificationTest {
            id: "AK".to_string(),
            name: "Eigenvalue-only blind inference cannot recover labeling".to_string(),
            passed: r.pass,
            detail: format!(
                "eigenRecovered={}/{} spectrumRecovered={}/{} scoreFlat={}",
                r.eigenvalue_recovered,
                r.total,
                r.spectrum_recovered,
                r.total,
                r.cases.iter().all(|c| c.score_spread < 1e-9)
            ),
        });
    }

    // R′ — RT deviation tracks excitation density (hold-out grid; tuning default is calibration only)
    {
        let cal = REFINEMENT_TUNING_DEFAULT;
        let r = run_rt_quench(&RtQuenchConfig {
            n: cal.n,
            field: cal.field,
            dt: cal.dt,
            steps: cal.steps_rt,
            seed: cal.seed,
        });
        let holdout = eval_r_prime_holdout();
        tests.push(FalsificationTest {
            id: "R′".to_string(),
            name: "RT slope deficit structured vs excitation density (prototype diagnostic)".to_string(),
            passed: holdout.all_passed(),
            detail: format!(
                "{}; cal devDensityCorr={:.3} structured={}",
                holdout.summary(),
                r.deviation_density_corr,
                r.structured_deviation
            ),
        });
    }

    // F′ — predictive early warning (hold-out grid; prototype diagnostic self-consistency)
    {
        let cal = REFINEMENT_TUNING_DEFAULT;
        let p = run_predictive_refinement(&AdaptiveRefinementConfig {
            n: cal.n,
            field: cal.field,
            dt: cal.dt,
            steps: cal.steps_adaptive,
            seed: cal.seed,
            delta_n: Some(2),
        });
        let holdout = eval_f_prime_holdout();
        tests.push(FalsificationTest {
            id: "F′".to_string(),
            name: "Predictive RT warning precedes failure (prototype diagnostic)".to_string(),
            passed: holdout.all_passed(),
            detail: format!(
                "{}; cal leadTime={} lateRecoverable={} warnStep={:?} failStep={:?}",
                holdout.summary(),
                p.lead_time,
                p.late_split_recoverable,
                p.early_warning_step,
                p.failure_step
            ),
        });
    }

    // C′ — geodesic deviation proxy suite (hold-out grid; prototype diagnostic)
    {
        let cal = REFINEMENT_TUNING_DEFAULT;
        let c = run_curvature_proxy_quench(&CurvatureProxyQuenchConfig {
            n: cal.n,
            field: cal.field,
            dt: cal.dt,
            steps: cal.steps_rt,
            seed: cal.seed,
            xi: 1.0,
        });
        let holdout = eval_c_prime_holdout();
        tests.push(FalsificationTest {
            id: "C′".to_string(),
            name: "Curvature proxy suite internally consistent (prototype diagnostic)".to_string(),
            passed: holdout.all_passed(),
            detail: format!(
                "{}; cal geoDensityCorr={:.3} crossCorr={:.3} consistent={}",
                holdout.summary(),
                c.geo_density_corr,
                c.proxy_correlation,
                c.internally_consistent
            ),
        });
    }

    // I′ — Pauli stabilizer generators (hold-out coupling/window grid; calibrated on coupling=0.9, window_radius=2)
    {
        let cal = run_stabilizer_search(&StabilizerSearchConfig {
            excitation: ExcitationSubspaceConfig {
                n: StabHoldoutCase::N,
                field: StabHoldoutCase::FIELD,
                dt: StabHoldoutCase::DT,
                steps: StabHoldoutCase::STEPS,
                couple_step: StabHoldoutCase::COUPLE_STEP,
                coupling: STAB_TUNING_DEFAULT.coupling,
                seed: StabHoldoutCase::SEED,
                window_radius: STAB_TUNING_DEFAULT.window_radius,
                model: None,
            },
        });
        let holdout = eval_i_prime_holdout();
        tests.push(FalsificationTest {
            id: "I′".to_string(),
            name: "Pauli stabilizer generators on branch subspace".to_string(),
            passed: holdout.all_passed(),
            detail: format!(
                "{}; cal generators={} distance={} scales={}",
                holdout.summary(),
                cal.stabilizer.generator_count,
                cal.stabilizer.code_distance,
                cal.distance_scales
            ),
        });
    }

    // S′ — defect localization longer-lived in ordered phase
    {
        let p = run_particle_stability_probe(&ParticleStabilityConfig {
            n: 10,
            dt: 0.2,
            steps: 24,
        });
        tests.push(FalsificationTest {
            id: "S′".to_string(),
            name: "Defect more localized in ordered vs disordered phase".to_string(),
            passed: p.inner.ordered_longer_lived,
            detail: format!(
                "ordered={:.3} disordered={:.3}",
                p.inner.ordered.localization_fraction, p.inner.disordered.localization_fraction
            ),
        });
    }

    // T′ — env branch weights track distinguishability (hold-out grid; calibrated on n=8, field=1.2, couple_step=7)
    {
        let cal = run_branch_born_probe(&BranchBornConfig {
            n: BORN_TUNING_DEFAULT.n,
            field: BORN_TUNING_DEFAULT.field,
            dt: BornHoldoutCase::DT,
            steps: BornHoldoutCase::STEPS,
            couple_step: BORN_TUNING_DEFAULT.couple_step,
        });
        let holdout = eval_t_prime_holdout();
        tests.push(FalsificationTest {
            id: "T′".to_string(),
            name: "Branch Born weights co-move with distinguishability".to_string(),
            passed: holdout.all_passed(),
            detail: format!(
                "{}; cal entropyCorr={:.3} imbalanceCorr={:.3}",
                holdout.summary(),
                cal.inner.entropy_overlap_corr,
                cal.inner.imbalance_overlap_corr
            ),
        });
    }

    // U′ — EFT DOF per site: two-of-three (branch rank, code rate, participation ratio)
    {
        let e = run_eft_dimension_probe(&EftDimensionConfig {
            excitation: ExcitationSubspaceConfig {
                n: 8,
                field: 1.2,
                dt: 0.2,
                steps: 22,
                couple_step: 7,
                coupling: 0.9,
                seed: 4242,
                window_radius: 2,
                model: None,
            },
        });
        tests.push(FalsificationTest {
            id: "U′".to_string(),
            name: "EFT DOF two-of-three: rank, code rate, participation ratio".to_string(),
            passed: e.dof_agreement,
            detail: format!(
                "meas={:.3} pred={:.3} indep={:.3} pairs mp={} mi={} pi={}",
                e.measured_dof_per_site,
                e.predicted_dof_per_site,
                e.independent_dof_per_site,
                e.measured_predicted_agree,
                e.measured_independent_agree,
                e.predicted_independent_agree,
            ),
        });
    }

    // V′ — in-place tensor split relieves pressure (hold-out grid; prototype diagnostic)
    {
        let cal = REFINEMENT_TUNING_DEFAULT;
        let v = run_inplace_split_probe(&InplaceSplitConfig {
            n: cal.n,
            field: cal.field,
            dt: cal.dt,
            steps: cal.steps_adaptive,
            seed: cal.seed,
            delta_n: Some(2),
        });
        let holdout = eval_v_prime_holdout();
        let cal_detail = v
            .inner
            .split_event
            .as_ref()
            .map(|e| {
                format!(
                    "step={} preP={:.3} inPlaceP={:.3} delta={:.3}",
                    e.trigger_step, e.pre.pressure, e.in_place.pressure, e.pressure_delta
                )
            })
            .unwrap_or_else(|| "no split trigger".to_string());
        tests.push(FalsificationTest {
            id: "V′".to_string(),
            name: "In-place tensor split relieves refinement pressure (prototype diagnostic)".to_string(),
            passed: holdout.all_passed(),
            detail: format!("{}; cal {}", holdout.summary(), cal_detail),
        });
    }

    // W′ — holographic signatures saturate at finite n on TFIM chain (blind locality on |ψ⟩)
    {
        let w = run_holographic_bound_probe(&HolographicBoundConfig {
            field: 1.5,
            n_min: 6,
            n_max: 12,
        });
        let passed = w.inner.n_min.is_some() && w.inner.bound_scales;
        let locality_at_nmin = w
            .inner
            .n_min
            .and_then(|n| w.inner.points.iter().find(|p| p.n == n))
            .map(|p| p.locality_fraction);
        tests.push(FalsificationTest {
            id: "W′".to_string(),
            name: "Holographic bound n_min finite with scaling plateau".to_string(),
            passed,
            detail: format!(
                "nMin={:?}, saturation={}, scales={}, blindLocalityAtNMin={}",
                w.inner.n_min,
                w.inner.n_saturation,
                w.inner.bound_scales,
                locality_at_nmin
                    .map(|l| format!("{l:.2}"))
                    .unwrap_or_else(|| "n/a".to_string())
            ),
        });
    }

    // X — code subspace identified: branch states inside, mixed state relatively outside
    {
        let q = run_qecc_probe(&StabilizerSearchConfig {
            excitation: ExcitationSubspaceConfig {
                n: 8,
                field: 1.2,
                dt: 0.2,
                steps: 22,
                couple_step: 7,
                coupling: 0.9,
                seed: 4242,
                window_radius: 2,
                model: None,
            },
        });
        let passed = q.qecc.code_subspace_found && q.qecc.fidelity_selectivity > 1.1;
        tests.push(FalsificationTest {
            id: "X".to_string(),
            name: "Code subspace identified: branches inside, mixed state less so".to_string(),
            passed,
            detail: format!(
                "{} k={} d={} b0={:.3} b1={:.3} mixed={:.3} sel={:.2}×",
                q.qecc.code_label,
                q.qecc.k_logical,
                q.qecc.d_distance,
                q.qecc.branch0_code_fidelity,
                q.qecc.branch1_code_fidelity,
                q.qecc.mixed_code_fidelity,
                q.qecc.fidelity_selectivity,
            ),
        });
    }

    // Y + Z — Poincaré composite (shared run): compute once, emit two tests
    {
        let pc = run_poincare_composite(&PoincareCompositeConfig {
            rows: 4,
            cols: 4,
            field: 1.2,
            dt: 0.2,
            steps: 28,
            n_chain: 16,
            chain_modes: 3,
            chain_field: 1.0,
            chain_dt: 0.15,
            chain_steps: 48,
        });

        // Y — multi-frame boost: 4 spatially distinct observer frames; extends P from 2 to 4 frames.
        tests.push(FalsificationTest {
            id: "Y".to_string(),
            name: "Multi-frame boost: light-cone speed consistent across 4 observer frames".to_string(),
            passed: pc.boost_ok,
            detail: format!(
                "boost_cv={:.3} frames=[{}]",
                pc.boost_cv,
                pc.boost_frames
                    .iter()
                    .map(|f| format!("{:.3}", f.velocity))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        });

        // Z — full Poincaré composite: rotation + multi-frame boost + dispersion all pass simultaneously.
        tests.push(FalsificationTest {
            id: "Z".to_string(),
            name: "Full Poincaré composite: rotation + boost + dispersion all pass".to_string(),
            passed: pc.all_poincare_ok,
            detail: format!(
                "{}/{} pillars; rot_cv={:.3} boost_cv={:.3} disp_r2={:.3}",
                pc.tests_passed,
                pc.tests_total,
                pc.rotation_cv,
                pc.boost_cv,
                pc.dispersion_linear_r2,
            ),
        });
    }

    // AA — locality from spectrum: blind MI+bandwidth inference recovers chain factorization
    {
        let ls = run_locality_spectrum_battery();
        let detail_cases: Vec<String> = ls
            .cases
            .iter()
            .map(|c| format!("{}×{}", if c.recovered { "✓" } else { "✗" }, c.seed))
            .collect();
        tests.push(FalsificationTest {
            id: "AA".to_string(),
            name: "Locality from spectrum: MI+bandwidth recovers chain factorization (no Ĥ)".to_string(),
            passed: ls.pass,
            detail: format!(
                "{}/{} recovered ({:.0}%): {}",
                ls.recovered,
                ls.total,
                ls.recovery_rate * 100.0,
                detail_cases.join(" ")
            ),
        });
    }


    // AL — AA-blind MI+bandwidth recovers 2D TFIM grid/torus (mod D₄); chain n=6 control
    {
        let b = run_aa_blind_2d_boundary();
        tests.push(FalsificationTest {
            id: "AL".to_string(),
            name: "AA-blind locality recovers 2D grid/torus and chain control".to_string(),
            passed: b.pass,
            detail: format!(
                "chain n=6 recovered={}; grid 3×3 recovered={}; torus 2×2 recovered={}",
                b.chain_recovered, b.grid_recovered, b.torus_recovered
            ),
        });
    }

    // AM — multi-clock G relational-time signature breaks on long chains (n≥11)
    {
        let b = run_multi_clock_g_scaling_boundary();
        tests.push(FalsificationTest {
            id: "AM".to_string(),
            name: "Multi-clock G signature breaks on long chains; n=9 control passes".to_string(),
            passed: b.pass,
            detail: format!(
                "n=9 G pass={} minPairwiseR2={:.3}; n=11 G fail={} minPairwiseR2={:.3}",
                b.n9_g_pass,
                b.n9_min_pairwise_r2,
                b.n11_g_fail,
                b.n11_min_pairwise_r2
            ),
        });
    }

    let passed = tests.iter().filter(|t| t.passed).count();
    FalsificationBatteryResult {
        total: tests.len(),
        passed,
        tests,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}
