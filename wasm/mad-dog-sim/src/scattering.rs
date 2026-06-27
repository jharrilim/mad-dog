//! Two-defect scattering — light-cone worldline tracking.

use crate::models::tfim_chain;
use crate::quantum::QuantumState;
use crate::relational_time::{evolve_trajectory, fit_affine};
use crate::spacetime::{build_light_cone_trajectory, SpacetimeConfig, SpacetimeSlice, WorldlinePoint};
use serde::{Deserialize, Serialize};

fn default_scatter_order() -> usize {
    4
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScatteringConfig {
    pub n: usize,
    pub field: f64,
    pub dt: f64,
    pub steps: usize,
    #[serde(default)]
    pub defect_sites: Option<[usize; 2]>,
    /// Omit full slice payload (metrics + worldlines only).
    #[serde(default)]
    pub lite: bool,
    #[serde(default = "default_scatter_order")]
    pub taylor_order: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScatteringWorldlinePoint {
    pub t: f64,
    pub site: usize,
    pub amplitude: f64,
}

impl From<WorldlinePoint> for ScatteringWorldlinePoint {
    fn from(p: WorldlinePoint) -> Self {
        Self {
            t: p.t,
            site: p.site,
            amplitude: p.amplitude,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScatteringResult {
    pub n: usize,
    pub field: f64,
    pub defect_sites: [usize; 2],
    pub slices: Vec<SpacetimeSlice>,
    pub worldlines: [Vec<ScatteringWorldlinePoint>; 2],
    pub separation_series: Vec<f64>,
    pub velocities: [f64; 2],
    pub both_moved: bool,
    pub crossed: bool,
    pub min_separation: f64,
    pub phase_series: Vec<f64>,
    pub post_interaction_phase_std: f64,
    pub phase_stable: bool,
    pub overlap_step: usize,
    pub overlap_detected: bool,
    pub interaction_phase_shift: f64,
    pub separation_time_delay: f64,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

fn fit_site_velocity(worldline: &[ScatteringWorldlinePoint], burn_in_frac: f64) -> f64 {
    let start = ((worldline.len() as f64) * burn_in_frac).floor() as usize;
    if worldline.len().saturating_sub(start) < 2 {
        return 0.0;
    }
    let mut sum_t = 0.0;
    let mut sum_s = 0.0;
    let mut sum_tt = 0.0;
    let mut sum_ts = 0.0;
    let mut count = 0.0;
    for p in worldline.iter().skip(start) {
        let t = p.t;
        let s = p.site as f64;
        sum_t += t;
        sum_s += s;
        sum_tt += t * t;
        sum_ts += t * s;
        count += 1.0;
    }
    let denom = count * sum_tt - sum_t * sum_t;
    if denom.abs() < 1e-12 {
        return 0.0;
    }
    (count * sum_ts - sum_t * sum_s) / denom
}

fn worldline_moved(worldline: &[ScatteringWorldlinePoint], initial: usize) -> bool {
    worldline
        .iter()
        .any(|p| (p.site as i32 - initial as i32).abs() >= 1)
}

fn analyze_crossing(
    left: &[ScatteringWorldlinePoint],
    right: &[ScatteringWorldlinePoint],
    d1: usize,
    d2: usize,
) -> (bool, f64, Vec<f64>) {
    let mut min_sep = f64::INFINITY;
    let mut crossed = false;
    let mut separation_series = Vec::with_capacity(left.len());
    let overlap_start = left.len() * 15 / 100;
    for (k, (l, r)) in left.iter().zip(right.iter()).enumerate() {
        let sep = (l.site as i32 - r.site as i32).unsigned_abs() as f64;
        separation_series.push(sep);
        if k >= overlap_start {
            min_sep = min_sep.min(sep);
            if d1 < d2 && l.site >= r.site {
                crossed = true;
            }
            if d1 > d2 && l.site <= r.site {
                crossed = true;
            }
        }
    }
    (
        crossed,
        if min_sep.is_finite() { min_sep } else { 0.0 },
        separation_series,
    )
}

fn amplitude_at(psi: &QuantumState, index: usize) -> (f64, f64) {
    (psi.data[2 * index], psi.data[2 * index + 1])
}

fn exchange_phase(psi: &QuantumState, d1: usize, d2: usize) -> f64 {
    let i00 = 0;
    let i10 = 1 << d1;
    let i01 = 1 << d2;
    let i11 = i10 | i01;
    let phases = [i00, i10, i01, i11].map(|idx| {
        let (re, im) = amplitude_at(psi, idx);
        im.atan2(re)
    });
    let delta = phases[3] + phases[0] - phases[1] - phases[2];
    // Wrap to [-π, π]
    let mut wrapped = delta;
    while wrapped > std::f64::consts::PI {
        wrapped -= std::f64::consts::TAU;
    }
    while wrapped < -std::f64::consts::PI {
        wrapped += std::f64::consts::TAU;
    }
    wrapped
}

fn unwrap_phases(phases: &[f64]) -> Vec<f64> {
    if phases.is_empty() {
        return Vec::new();
    }
    let mut out = vec![phases[0]];
    let mut offset = 0.0;
    for i in 1..phases.len() {
        let mut d = phases[i] - phases[i - 1];
        while d > std::f64::consts::PI {
            d -= std::f64::consts::TAU;
        }
        while d < -std::f64::consts::PI {
            d += std::f64::consts::TAU;
        }
        offset += d;
        out.push(phases[0] + offset);
    }
    out
}

fn track_exchange_phases(
    hamiltonian: &crate::quantum::Hamiltonian,
    initial: &QuantumState,
    reference: &QuantumState,
    dt: f64,
    steps: usize,
    order: usize,
    d1: usize,
    d2: usize,
) -> (Vec<f64>, f64, bool) {
    let trajectory = evolve_trajectory(hamiltonian, initial, Some(reference), dt, steps, order);
    let phase_series: Vec<f64> = trajectory
        .iter()
        .map(|p| exchange_phase(&p.psi, d1, d2))
        .collect();
    let start = phase_series.len() * 60 / 100;
    let unwrapped = unwrap_phases(&phase_series[start..]);
    if unwrapped.len() < 3 {
        return (phase_series, f64::INFINITY, false);
    }
    let times: Vec<f64> = (0..unwrapped.len()).map(|i| i as f64).collect();
    let (slope, intercept, fit_r2) = fit_affine(&times, &unwrapped);
    let residual_std = {
        let res: Vec<f64> = unwrapped
            .iter()
            .zip(times.iter())
            .map(|(&p, &t)| p - (slope * t + intercept))
            .collect();
        let rmean = res.iter().sum::<f64>() / res.len() as f64;
        let var = res.iter().map(|r| (r - rmean).powi(2)).sum::<f64>() / res.len() as f64;
        var.sqrt()
    };
    (phase_series, residual_std, fit_r2 > 0.85 && residual_std < 0.55)
}

/// Overlap-localized phase shift and separation time delay (Phase 11).
fn analyze_interaction(
    separation_series: &[f64],
    phase_series: &[f64],
    min_separation: f64,
    dt: f64,
) -> (usize, bool, f64, f64) {
    let n = separation_series.len();
    if n == 0 {
        return (0, false, f64::NAN, f64::NAN);
    }

    let burn_in = n * 15 / 100;
    let overlap_step = (burn_in..n)
        .min_by(|&a, &b| {
            separation_series[a]
                .partial_cmp(&separation_series[b])
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or(burn_in);

    let overlap_detected = min_separation < separation_series[0] - 1.0;

    const PRE_MARGIN: usize = 3;
    let pre_end = overlap_step.saturating_sub(PRE_MARGIN);
    let interaction_phase_shift = if pre_end >= 2 && overlap_step < phase_series.len() {
        let unwrapped_pre = unwrap_phases(&phase_series[..pre_end]);
        if unwrapped_pre.len() >= 2 {
            let times: Vec<f64> = (0..unwrapped_pre.len()).map(|i| i as f64).collect();
            let (slope, intercept, _) = fit_affine(&times, &unwrapped_pre);
            let unwrapped_to_overlap = unwrap_phases(&phase_series[..=overlap_step]);
            let actual = *unwrapped_to_overlap.last().unwrap_or(&f64::NAN);
            actual - (slope * overlap_step as f64 + intercept)
        } else {
            f64::NAN
        }
    } else {
        f64::NAN
    };

    let early_end = (n * 20 / 100).max(2).min(n);
    let early_sep = &separation_series[..early_end];
    let early_times: Vec<f64> = (0..early_sep.len()).map(|i| i as f64).collect();
    let separation_time_delay = if early_sep.len() >= 2 {
        let (slope, intercept, _) = fit_affine(&early_times, early_sep);
        if slope.abs() < 1e-12 {
            f64::NAN
        } else {
            let k_pred = (min_separation - intercept) / slope;
            if k_pred.is_finite() {
                (overlap_step as f64 - k_pred) * dt
            } else {
                f64::NAN
            }
        }
    } else {
        f64::NAN
    };

    (
        overlap_step,
        overlap_detected,
        interaction_phase_shift,
        separation_time_delay,
    )
}

pub fn run_two_defect_scattering(config: &ScatteringConfig) -> ScatteringResult {
    let d1 = config.defect_sites.map(|s| s[0]).unwrap_or(3);
    let d2 = config.defect_sites.map(|s| s[1]).unwrap_or(config.n - 4);
    let model = tfim_chain(config.n, 1.0, config.field);

    let mut initial = QuantumState::zero(config.n);
    initial.data[2 * (1 << d1)] = 1.0;
    initial.data[2 * (1 << d2)] = 1.0;
    let reference = {
        let mut r = QuantumState::zero(config.n);
        r.data[0] = 1.0;
        r
    };

    let lite = config.lite;
    let initial_for_phase = initial.clone_state();
    let reference_for_phase = reference.clone_state();
    let spacetime = build_light_cone_trajectory(SpacetimeConfig {
        hamiltonian: &model.hamiltonian,
        initial,
        reference: Some(reference),
        dt: config.dt,
        steps: config.steps,
        embed_dim: 1,
        align_to: None,
        order: config.taylor_order,
        include_geometry: false,
        track_energy: !lite,
        retain_slices: !lite,
        worldline_defects: Some([d1, d2]),
        track_worldline: false,
        defect_site: None,
        seed: 42,
        signal_reference_site: None,
    });

    let [raw1, raw2] = spacetime
        .worldlines
        .expect("worldlines tracked during scattering");
    let wl1: Vec<ScatteringWorldlinePoint> = raw1
        .into_iter()
        .map(ScatteringWorldlinePoint::from)
        .collect();
    let wl2: Vec<ScatteringWorldlinePoint> = raw2
        .into_iter()
        .map(ScatteringWorldlinePoint::from)
        .collect();
    let (crossed, min_separation, separation_series) = analyze_crossing(&wl1, &wl2, d1, d2);
    let velocities = [
        fit_site_velocity(&wl1, 0.15),
        fit_site_velocity(&wl2, 0.15),
    ];
    let both_moved = worldline_moved(&wl1, d1) && worldline_moved(&wl2, d2);
    let (phase_series, post_interaction_phase_std, phase_stable) = track_exchange_phases(
        &model.hamiltonian,
        &initial_for_phase,
        &reference_for_phase,
        config.dt,
        config.steps,
        config.taylor_order,
        d1,
        d2,
    );
    let (overlap_step, overlap_detected, interaction_phase_shift, separation_time_delay) =
        analyze_interaction(
            &separation_series,
            &phase_series,
            min_separation,
            config.dt,
        );

    ScatteringResult {
        n: config.n,
        field: config.field,
        defect_sites: [d1, d2],
        slices: if lite {
            Vec::new()
        } else {
            spacetime.slices
        },
        worldlines: [wl1, wl2],
        separation_series,
        velocities,
        both_moved,
        crossed,
        min_separation,
        phase_series,
        post_interaction_phase_std,
        phase_stable,
        overlap_step,
        overlap_detected,
        interaction_phase_shift,
        separation_time_delay,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> ScatteringConfig {
        ScatteringConfig {
            n: 12,
            field: 0.7,
            dt: 0.12,
            steps: 40,
            defect_sites: Some([3, 8]),
            lite: true,
            taylor_order: 4,
        }
    }

    #[test]
    fn interaction_metrics_on_default_config() {
        let r = run_two_defect_scattering(&default_config());
        assert!(r.overlap_detected, "cones should close on default demo");
        assert!(
            r.overlap_step > 0 && r.overlap_step < r.separation_series.len(),
            "overlap_step={} out of range",
            r.overlap_step
        );
        assert!(
            r.interaction_phase_shift.is_finite(),
            "interaction_phase_shift should be finite"
        );
        assert!(
            r.separation_time_delay.is_finite(),
            "separation_time_delay should be finite"
        );
    }

    #[test]
    fn interaction_phase_shift_exceeds_ad_threshold() {
        let r = run_two_defect_scattering(&default_config());
        assert!(
            r.interaction_phase_shift.abs() > 0.05,
            "AD threshold: |phase_shift|={:.4}",
            r.interaction_phase_shift
        );
    }

    #[test]
    fn analyze_interaction_handles_empty_series() {
        let (step, detected, shift, delay) = analyze_interaction(&[], &[], 0.0, 0.1);
        assert_eq!(step, 0);
        assert!(!detected);
        assert!(shift.is_nan());
        assert!(delay.is_nan());
    }
}
