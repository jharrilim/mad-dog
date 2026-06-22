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
use crate::run_refinement::{
    run_adaptive_refinement, run_predictive_refinement, run_refinement_quench,
    AdaptiveRefinementConfig, RefinementQuenchConfig,
};
use crate::run_matter::{
    run_branch_born_probe, run_eft_dimension_probe, run_particle_stability_probe,
    run_stabilizer_search, BranchBornConfig, EftDimensionConfig, ParticleStabilityConfig,
    StabilizerSearchConfig,
};
use crate::run_factor_dynamics::{
    run_holographic_bound_probe, run_inplace_split_probe,
};
use crate::holographic_bound::HolographicBoundConfig;
use crate::tensor_split::InplaceSplitConfig;
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
use crate::dispersion::{run_dispersion, DispersionConfig};
use crate::lorentz::grid_cardinal_speed_cv;
use crate::run_lorentz_scaling::run_lorentz_scaling;
use crate::scattering::{run_two_defect_scattering, ScatteringConfig};
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
            n: 12,
            field: 0.7,
            dt: 0.12,
            steps: 40,
            defect_sites: Some([3, 8]),
            lite: true,
            taylor_order: 4,
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

    // M — negative controls (random nonlocal + scrambled spectrum)
    {
        let random = run_factorization_search(&FactorizationSearchConfig {
            kind: "random".to_string(),
            n: 6,
            field: 1.5,
            seed: 777,
            top_k: 3,
            input_mode: Some("spectrum".to_string()),
            search_method: Some("exact".to_string()),
            eigenstate_count: Some(3),
            graph_kind: None,
            rows: None,
            cols: None,
            distance_decay: None,
            annealing_steps: None,
            spectrum_scramble: None,
        });
        let scrambled = run_factorization_search(&FactorizationSearchConfig {
            kind: "shuffled_chain".to_string(),
            n: 6,
            field: 1.5,
            seed: 4242,
            top_k: 3,
            input_mode: Some("spectrum".to_string()),
            search_method: Some("exact".to_string()),
            eigenstate_count: Some(3),
            graph_kind: None,
            rows: None,
            cols: None,
            distance_decay: None,
            annealing_steps: None,
            spectrum_scramble: Some(true),
        });
        let m1 = !random.recovered_identity;
        let m2 = !scrambled.recovered_identity;
        tests.push(FalsificationTest {
            id: "M".to_string(),
            name: "Negative controls reject fake locality".to_string(),
            passed: m1 && m2,
            detail: format!(
                "randomRecovered={} scrambledRecovered={} (both should be false)",
                random.recovered_identity, scrambled.recovered_identity
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
            n: 12,
            field: 0.7,
            dt: 0.12,
            steps: 40,
            defect_sites: Some([3, 8]),
            lite: true,
            taylor_order: 4,
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

    // I′ — Pauli stabilizer generators on branch excitation window
    {
        let s = run_stabilizer_search(&StabilizerSearchConfig {
            excitation: ExcitationSubspaceConfig {
                n: 8,
                field: 1.2,
                dt: 0.2,
                steps: 22,
                couple_step: 7,
                coupling: 0.9,
                seed: 4242,
                window_radius: 2,
            },
        });
        tests.push(FalsificationTest {
            id: "I′".to_string(),
            name: "Pauli stabilizer generators on branch subspace".to_string(),
            passed: s.stabilizer.stabilizer_found && s.distance_scales,
            detail: format!(
                "generators={}, distance={}, scales={}",
                s.stabilizer.generator_count, s.stabilizer.code_distance, s.distance_scales
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

    // T′ — env branch weights track distinguishability
    {
        let b = run_branch_born_probe(&BranchBornConfig {
            n: 8,
            field: 1.2,
            dt: 0.2,
            steps: 22,
            couple_step: 7,
        });
        tests.push(FalsificationTest {
            id: "T′".to_string(),
            name: "Branch Born weights co-move with distinguishability".to_string(),
            passed: b.inner.born_consistent,
            detail: format!(
                "entropyCorr={:.3}, imbalanceCorr={:.3}",
                b.inner.entropy_overlap_corr, b.inner.imbalance_overlap_corr
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

    let passed = tests.iter().filter(|t| t.passed).count();
    FalsificationBatteryResult {
        total: tests.len(),
        passed,
        tests,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}
