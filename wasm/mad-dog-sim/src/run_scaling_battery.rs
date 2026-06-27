//! Phase 12 scaling battery — fast regression checks for scaling research probes.

use crate::factorization::UniquenessReport;
use crate::locality_spectrum::{blind_lattice_case, blind_case};
use crate::run_factorization::{run_factorization_search, FactorizationSearchConfig};
use crate::multi_clock::MultiClockResult;
use crate::run_multi_clock::{run_multi_clock, MultiClockRunConfig};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScalingCheck {
    pub id: String,
    pub pass: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScalingBatteryResult {
    pub pass: bool,
    pub checks: Vec<ScalingCheck>,
    pub elapsed_ms: f64,
}


fn multiclock_chain_config(n: usize) -> MultiClockRunConfig {
    MultiClockRunConfig {
        kind: "chain".to_string(),
        n: Some(n),
        rows: None,
        cols: None,
        lz: None,
        field: 1.0,
        dt: 0.2,
        steps: 40,
        clock_sites: None,
        physical_slices: Some(15),
    }
}

fn g_criterion_pass(m: &MultiClockResult) -> bool {
    m.defect_uniform_r2 > 0.95 && m.min_pairwise_r2 < 0.95
}

fn push_multiclock_chain_g(checks: &mut Vec<ScalingCheck>, n: usize, expect_g_pass: bool) {
    let m = run_multi_clock(&multiclock_chain_config(n));
    let g_pass = g_criterion_pass(&m);
    let pass = if expect_g_pass { g_pass } else { !g_pass };
    let suffix = if expect_g_pass {
        String::new()
    } else {
        "_g_breaks".to_string()
    };
    checks.push(ScalingCheck {
        id: format!("multiclock_chain_n{n}{suffix}"),
        pass,
        detail: format!(
            "expect {}: defectUniformR2={:.3} minPairwiseR2={:.3} inconsistentPairs={}",
            if expect_g_pass { "G pass" } else { "G fail" },
            m.defect_uniform_r2,
            m.min_pairwise_r2,
            m.inconsistent_pairs
        ),
    });
}

/// Phase 12 falsification AM: G relational-time signature holds at n=9 but breaks at n≥11.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiClockGScalingBoundaryResult {
    pub pass: bool,
    pub n9_g_pass: bool,
    pub n11_g_fail: bool,
    pub n9_min_pairwise_r2: f64,
    pub n11_min_pairwise_r2: f64,
}

pub fn run_multi_clock_g_scaling_boundary() -> MultiClockGScalingBoundaryResult {
    let n9 = run_multi_clock(&multiclock_chain_config(9));
    let n11 = run_multi_clock(&multiclock_chain_config(11));
    let n9_g_pass = g_criterion_pass(&n9);
    let n11_g_fail = n11.min_pairwise_r2 >= 0.95;
    MultiClockGScalingBoundaryResult {
        pass: n9_g_pass && n11_g_fail,
        n9_g_pass,
        n11_g_fail,
        n9_min_pairwise_r2: n9.min_pairwise_r2,
        n11_min_pairwise_r2: n11.min_pairwise_r2,
    }
}

fn push_aa_blind_chain(checks: &mut Vec<ScalingCheck>, n: usize) {
    let c = blind_case("tfim", n, 1.5, 4242);
    checks.push(ScalingCheck {
        id: format!("aa_blind_chain_n{n}"),
        pass: c.recovered,
        detail: format!(
            "recovered={} score={:.3} miNn={:.3}",
            c.recovered, c.score, c.mi_nn_ratio
        ),
    });
}

fn uniqueness_pass(u: &UniquenessReport, min_gap: f64, max_classes: usize) -> bool {
    u.true_in_top_k
        && u.best_class_size <= 2
        && u.equivalence_class_count <= max_classes
        && u.score_gap_to_second_class >= min_gap
}

fn push_uniqueness_spectrum(
    checks: &mut Vec<ScalingCheck>,
    id: &str,
    kind: &str,
    n: usize,
    seed: u32,
    min_gap: f64,
) {
    let eigen = if n <= 4 { 2 } else { 3 };
    let r = run_factorization_search(&FactorizationSearchConfig {
        kind: kind.to_string(),
        n,
        field: 1.5,
        seed,
        top_k: 5,
        input_mode: Some("spectrum".to_string()),
        search_method: Some("exact".to_string()),
        eigenstate_count: Some(eigen),
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
    let pass = r.recovered_identity
        && r.uniqueness
            .as_ref()
            .map(|u| uniqueness_pass(u, min_gap, 3))
            .unwrap_or(false);
    let detail = match &r.uniqueness {
        Some(u) => format!(
            "recovered={} classes={} bestSize={} gap={:.3} trueInTopK={}",
            r.recovered_identity,
            u.equivalence_class_count,
            u.best_class_size,
            u.score_gap_to_second_class,
            u.true_in_top_k
        ),
        None => format!("recovered={} (no uniqueness report)", r.recovered_identity),
    };
    checks.push(ScalingCheck {
        id: id.to_string(),
        pass,
        detail,
    });
}

/// Key scaling probes from Phase 12 research (target runtime under ~2 min).
pub fn run_scaling_battery() -> ScalingBatteryResult {
    let mut checks = Vec::new();

    let mut gap_n6 = None;
    let mut gap_n8 = None;

    for n in [4usize, 6, 8] {
        let eigen = if n <= 4 { 2 } else { 3 };
        let r = run_factorization_search(&FactorizationSearchConfig {
            kind: "shuffled_chain".to_string(),
            n,
            field: 1.5,
            seed: 4242,
            top_k: 5,
            input_mode: Some("spectrum".to_string()),
            search_method: Some("exact".to_string()),
            eigenstate_count: Some(eigen),
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
        let gap = r
            .uniqueness
            .as_ref()
            .map(|u| u.score_gap_to_second_class)
            .unwrap_or(-1.0);
        if n == 6 {
            gap_n6 = Some(gap);
        } else if n == 8 {
            gap_n8 = Some(gap);
        }
        checks.push(ScalingCheck {
            id: format!("factorization_spectrum_chain_n{n}"),
            pass: r.recovered_identity,
            detail: format!(
                "recovered={} score={:.3} gap={:.3}",
                r.recovered_identity, r.best.score, gap
            ),
        });
    }

    for n in [4usize, 6, 8] {
        push_aa_blind_chain(&mut checks, n);
    }

    if let (Some(g6), Some(g8)) = (gap_n6, gap_n8) {
        checks.push(ScalingCheck {
            id: "uniqueness_gap_n6_vs_n8".to_string(),
            pass: g6 > 0.10 && g8 > 0.06 && g8 < g6,
            detail: format!("gap6={:.3} gap8={:.3} (narrows, stays positive)", g6, g8),
        });
    }

    push_uniqueness_spectrum(
        &mut checks,
        "uniqueness_spectrum_n4",
        "shuffled_chain",
        4,
        4242,
        0.15,
    );
    push_uniqueness_spectrum(
        &mut checks,
        "uniqueness_xx_n6",
        "shuffled_xx_chain",
        6,
        4242,
        0.08,
    );
    push_uniqueness_spectrum(
        &mut checks,
        "uniqueness_sparse_n6",
        "shuffled_sparse_chain",
        6,
        4242,
        0.004,
    );

    let grid = blind_lattice_case("grid", 3, 3, 1.5, 4242);
    checks.push(ScalingCheck {
        id: "aa_blind_grid_3x3".to_string(),
        pass: !grid.recovered,
        detail: format!(
            "expect fail: recovered={} score={:.3} miNn={:.3}",
            grid.recovered, grid.score, grid.mi_nn_ratio
        ),
    });

    let torus = blind_lattice_case("torus", 2, 2, 1.5, 4242);
    checks.push(ScalingCheck {
        id: "aa_blind_torus_2x2".to_string(),
        pass: !torus.recovered,
        detail: format!(
            "expect fail: recovered={} score={:.3} miNn={:.3}",
            torus.recovered, torus.score, torus.mi_nn_ratio
        ),
    });

    for n in [7usize, 9] {
        push_multiclock_chain_g(&mut checks, n, true);
    }
    push_multiclock_chain_g(&mut checks, 11, false);

    let pass = checks.iter().all(|c| c.pass);
    ScalingBatteryResult {
        pass,
        checks,
        elapsed_ms: 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaling_battery_passes() {
        let r = run_scaling_battery();
        assert!(
            r.pass,
            "scaling battery failed:\n{}",
            r.checks
                .iter()
                .filter(|c| !c.pass)
                .map(|c| format!("  {} — {}", c.id, c.detail))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    #[test]
    fn multi_clock_g_scaling_boundary() {
        let b = run_multi_clock_g_scaling_boundary();
        assert!(
            b.pass,
            "n9_g_pass={} n11_g_fail={} n9_minPairwiseR2={:.3} n11_minPairwiseR2={:.3}",
            b.n9_g_pass,
            b.n11_g_fail,
            b.n9_min_pairwise_r2,
            b.n11_min_pairwise_r2
        );
    }
}
