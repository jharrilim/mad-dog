//! Falsification battery — Mad-Dog-specific claim tests (WASM-only).

use crate::geometry::{entropy_of_region, mutual_information_matrix};
use crate::models::{line_locality_fraction, tfim_chain, tfim_torus};
use crate::quantum::ground_state;
use crate::rng::Rng;
use crate::run_factorization::{run_factorization_search, FactorizationSearchConfig};
use crate::run_light_cone_compare::{run_light_cone_compare, LightConeCompareConfig};
use crate::modular_time::ModularDualClockConfig;
use crate::run_modular_dual_clock::run_modular_dual_clock;
use crate::run_refinement::{
    run_adaptive_refinement, run_refinement_quench, AdaptiveRefinementConfig,
    RefinementQuenchConfig,
};
use crate::run_excitation_subspace::{run_excitation_subspace_probe, ExcitationSubspaceConfig};
use crate::run_geometry_stability::{run_geometry_stability, GeometryStabilityConfig};
use crate::run_multi_clock::{run_multi_clock, MultiClockRunConfig};
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
            n,
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

    // J — MI distance ranking stable without Procrustes (ordered phase)
    {
        let ordered = run_geometry_stability(&GeometryStabilityConfig {
            n: 10,
            field: 1.2,
            dt: 0.2,
            steps: 20,
            xi: 1.0,
            seed: 42,
        });
        tests.push(FalsificationTest {
            id: "J".to_string(),
            name: "Gauge-free MI geometry stable in ordered phase".to_string(),
            passed: ordered.geometry_stable,
            detail: format!(
                "drift={:.3} corr={:.3} dimStd={:.3}",
                ordered.mean_distance_drift,
                ordered.mean_rank_correlation,
                ordered.dim_std
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
