//! Adaptive holographic refinement — port of `src/sim/refinement.ts`.

use crate::geometry::{analyze_emergent_geometry, entropy_of_region};
use crate::holography::{analyze_rt_relation, RtFit};
use crate::models::tfim_chain;
use crate::quantum::{ground_state, make_random_state, QuantumState};
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefinementBaselines {
    pub ground_area_growth: f64,
    pub random_area_growth: f64,
}

#[derive(Clone, Debug)]
pub struct RefinementThresholds {
    pub pressure: f64,
    pub min_rt_r2: f64,
    pub max_rt_slope_dev: f64,
}

impl Default for RefinementThresholds {
    fn default() -> Self {
        Self {
            pressure: 0.42,
            min_rt_r2: 0.82,
            max_rt_slope_dev: 0.35,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefinementReasons {
    pub rt_fit: bool,
    pub rt_slope: bool,
    pub area_law: bool,
    pub emergent_dim: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefinementDiagnostics {
    pub n: usize,
    pub rt_slope: f64,
    pub rt_r2: f64,
    pub emergent_dim: usize,
    pub area_growth: f64,
    pub area_pressure: f64,
    pub pressure: f64,
    pub needs_refinement: bool,
    pub reasons: RefinementReasons,
}

fn clamp01(x: f64) -> f64 {
    x.clamp(0.0, 1.0)
}

fn edge_anchored_entropies(state: &QuantumState) -> Vec<f64> {
    let n = state.n;
    let mut out = Vec::new();
    for l in 1..n {
        let region: Vec<usize> = (0..l).collect();
        out.push(entropy_of_region(state, &region));
    }
    out
}

pub fn area_growth_ratio(state: &QuantumState) -> f64 {
    let ent = edge_anchored_entropies(state);
    if ent.is_empty() {
        return 0.0;
    }
    let s1 = ent[0];
    let mid = ent[ent.len() / 2];
    mid / (s1 + 1e-9)
}

pub fn compute_refinement_baselines(n: usize, field: f64, seed: u32) -> RefinementBaselines {
    let model = tfim_chain(n, 1.0, field);
    let (ground, _, _) = ground_state(&model.hamiltonian, &mut Rng::new(seed), 4000, 1e-9);
    let random = make_random_state(n, &mut Rng::new(seed.wrapping_add(4242)));
    RefinementBaselines {
        ground_area_growth: area_growth_ratio(&ground),
        random_area_growth: area_growth_ratio(&random),
    }
}

pub fn defect_initial_state(n: usize) -> QuantumState {
    let center = n / 2;
    let mut initial = QuantumState::zero(n);
    initial.data[2 * (1 << center)] = 1.0;
    initial
}

pub fn measure_refinement_diagnostics(
    state: &QuantumState,
    expected_dim: usize,
    baselines: &RefinementBaselines,
    thresholds: &RefinementThresholds,
) -> RefinementDiagnostics {
    let RtFit {
        rt_slope,
        rt_r2,
        ..
    } = analyze_rt_relation(state, None);
    let report = analyze_emergent_geometry(state, 1.0, 3);
    let emergent_dim = report.mds.emergent_dim;
    let area_growth = area_growth_ratio(state);

    let span = baselines.random_area_growth - baselines.ground_area_growth + 1e-9;
    let area_pressure = clamp01((area_growth - baselines.ground_area_growth) / span);

    if area_growth < 1e-4 && rt_r2 < 1e-4 {
        return RefinementDiagnostics {
            n: state.n,
            rt_slope: 1.0,
            rt_r2: 1.0,
            emergent_dim,
            area_growth,
            area_pressure: 0.0,
            pressure: 0.0,
            needs_refinement: false,
            reasons: RefinementReasons {
                rt_fit: false,
                rt_slope: false,
                area_law: false,
                emergent_dim: false,
            },
        };
    }

    let rt_slope_safe = if rt_slope.is_finite() { rt_slope } else { 1.0 };
    let rt_r2_safe = if rt_r2.is_finite() { rt_r2 } else { 0.0 };

    let rt_deficit = clamp01(1.0 - rt_r2_safe);
    let slope_deficit = clamp01((rt_slope_safe - 1.0).abs() / thresholds.max_rt_slope_dev);
    let dim_pressure = clamp01(
        (emergent_dim as f64 - expected_dim as f64) / (expected_dim as f64 + 0.5).max(1.0),
    );

    let pressure = clamp01(
        0.35 * rt_deficit + 0.25 * slope_deficit + 0.25 * area_pressure + 0.15 * dim_pressure,
    );

    let reasons = RefinementReasons {
        rt_fit: rt_r2_safe < thresholds.min_rt_r2,
        rt_slope: (rt_slope_safe - 1.0).abs() > thresholds.max_rt_slope_dev,
        area_law: area_pressure > 0.55,
        emergent_dim: emergent_dim as f64 > expected_dim as f64 + 0.5,
    };

    let needs_refinement = pressure >= thresholds.pressure
        || (reasons.rt_fit && reasons.area_law)
        || (reasons.rt_slope && reasons.emergent_dim);

    RefinementDiagnostics {
        n: state.n,
        rt_slope: rt_slope_safe,
        rt_r2: rt_r2_safe,
        emergent_dim,
        area_growth,
        area_pressure,
        pressure,
        needs_refinement,
        reasons,
    }
}
