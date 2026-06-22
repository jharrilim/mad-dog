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

fn trace_slope_deficit(d: &RefinementDiagnostics, thresholds: &RefinementThresholds) -> f64 {
    clamp01((d.rt_slope - 1.0).abs() / thresholds.max_rt_slope_dev)
}

/// Clock-step lag between peak slope deficit and peak area pressure.
pub fn refinement_decoupling_lag(steps_and_diag: &[(usize, RefinementDiagnostics)]) -> usize {
    if steps_and_diag.is_empty() {
        return 0;
    }
    let thresholds = RefinementThresholds::default();
    let mut slope_peak = 0;
    let mut area_peak = 0;
    let mut max_slope = -1.0;
    let mut max_area = -1.0;
    for &(step, ref d) in steps_and_diag {
        let slope = trace_slope_deficit(d, &thresholds);
        if slope > max_slope {
            max_slope = slope;
            slope_peak = step;
        }
        if d.area_pressure > max_area {
            max_area = d.area_pressure;
            area_peak = step;
        }
    }
    slope_peak.abs_diff(area_peak)
}

/// Site index where edge-anchored entropy grows fastest — toy split hint.
pub fn suggest_split_site(state: &QuantumState) -> usize {
    let ent = edge_anchored_entropies(state);
    if ent.len() < 2 {
        return state.n / 2;
    }
    let mut max_jump = 0.0_f64;
    let mut site = state.n / 2;
    for i in 1..ent.len() {
        let jump = (ent[i] - ent[i - 1]).abs();
        if jump > max_jump {
            max_jump = jump;
            site = i;
        }
    }
    site
}

/// First step where RT fit degrades but overall refinement has not yet failed.
pub fn first_early_warning_step(
    steps_and_diag: &[(usize, RefinementDiagnostics)],
    thresholds: &RefinementThresholds,
) -> Option<usize> {
    let failure = first_failure_step(steps_and_diag);
    let mut peak_r2 = 0.0_f64;
    for &(step, ref d) in steps_and_diag {
        if failure.is_some_and(|f| step >= f) {
            break;
        }
        peak_r2 = peak_r2.max(d.rt_r2);
        let r2_drop = peak_r2 - d.rt_r2;
        if d.reasons.rt_fit || d.reasons.rt_slope {
            return Some(step);
        }
        if (r2_drop > 0.035 || d.rt_r2 < 0.95) && d.pressure < thresholds.pressure {
            return Some(step);
        }
    }
    None
}

/// First step where full refinement failure is flagged.
pub fn first_failure_step(steps_and_diag: &[(usize, RefinementDiagnostics)]) -> Option<usize> {
    steps_and_diag
        .iter()
        .find(|(_, d)| d.needs_refinement)
        .map(|(step, _)| *step)
}

/// Clock steps between early RT warning and full refinement failure.
pub fn early_warning_lead_time(warning: Option<usize>, failure: Option<usize>) -> usize {
    match (warning, failure) {
        (Some(w), Some(f)) if f >= w => f - w,
        _ => 0,
    }
}

/// Split trigger at peak-pressure step when the quench ends in refinement failure.
pub fn first_split_trigger_from_diag(
    steps: &[(usize, RefinementDiagnostics)],
    _thresholds: &RefinementThresholds,
) -> Option<usize> {
    let last = &steps.last()?.1;
    if !last.needs_refinement {
        return None;
    }
    steps
        .iter()
        .max_by(|a, b| {
            a.1.pressure
                .partial_cmp(&b.1.pressure)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(step, _)| *step)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitEvent {
    pub trigger_step: usize,
    pub trigger_t: f64,
    pub pre_n: usize,
    pub post_n: usize,
    pub suggested_split_site: usize,
    pub pre: RefinementDiagnostics,
    pub post: RefinementDiagnostics,
    pub accepted: bool,
    pub pressure_delta: f64,
}
