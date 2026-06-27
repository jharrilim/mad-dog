//! Phase 12 scaling battery — fast regression checks for scaling research probes.

use crate::locality_spectrum::{blind_lattice_case, blind_case};
use crate::run_factorization::{run_factorization_search, FactorizationSearchConfig};
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

fn push_factorization_chain(checks: &mut Vec<ScalingCheck>, n: usize) {
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
    checks.push(ScalingCheck {
        id: format!("factorization_spectrum_chain_n{n}"),
        pass: r.recovered_identity,
        detail: format!(
            "recovered={} score={:.3} gap={:.3}",
            r.recovered_identity,
            r.best.score,
            r.uniqueness
                .as_ref()
                .map(|u| u.score_gap_to_second_class)
                .unwrap_or(-1.0)
        ),
    });
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

/// Key scaling probes from Phase 12 research (target runtime under ~2 min).
pub fn run_scaling_battery() -> ScalingBatteryResult {
    let mut checks = Vec::new();

    for n in [4usize, 6, 8] {
        push_factorization_chain(&mut checks, n);
    }

    for n in [4usize, 6, 8] {
        push_aa_blind_chain(&mut checks, n);
    }

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
    let mc_pass = m.defect_uniform_r2 > 0.95 && m.min_pairwise_r2 < 0.95;
    checks.push(ScalingCheck {
        id: "multiclock_chain_n9".to_string(),
        pass: mc_pass,
        detail: format!(
            "defectUniformR2={:.3} minPairwiseR2={:.3}",
            m.defect_uniform_r2, m.min_pairwise_r2
        ),
    });

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
}
