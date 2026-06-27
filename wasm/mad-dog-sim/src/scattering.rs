//! Two-defect scattering — light-cone worldline tracking (chain, grid, cube).

use crate::dispersion::{build_dispersion, DispersionConfig};
use crate::models::{tfim_chain, tfim_cube, tfim_grid, BuiltModel, TruePosition};
use crate::quantum::{Hamiltonian, QuantumState};
use crate::relational_time::{evolve_trajectory, fit_affine};
use crate::spacetime::{build_light_cone_trajectory, SpacetimeConfig, SpacetimeSlice, WorldlinePoint};
use serde::{Deserialize, Serialize};

fn default_scatter_order() -> usize {
    4
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScatteringConfig {
    #[serde(default)]
    pub n: usize,
    pub field: f64,
    pub dt: f64,
    pub steps: usize,
    #[serde(default)]
    pub defect_sites: Option<[usize; 2]>,
    /// `chain` (default), `grid`, or `cube`.
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub rows: Option<usize>,
    #[serde(default)]
    pub cols: Option<usize>,
    #[serde(default)]
    pub lx: Option<usize>,
    #[serde(default)]
    pub ly: Option<usize>,
    #[serde(default)]
    pub lz: Option<usize>,
    /// Omit full slice payload (metrics + worldlines only).
    #[serde(default)]
    pub lite: bool,
    #[serde(default = "default_scatter_order")]
    pub taylor_order: usize,
}

impl Default for ScatteringConfig {
    fn default() -> Self {
        Self {
            n: 12,
            field: 0.7,
            dt: 0.12,
            steps: 40,
            defect_sites: None,
            kind: None,
            rows: None,
            cols: None,
            lx: None,
            ly: None,
            lz: None,
            lite: false,
            taylor_order: default_scatter_order(),
        }
    }
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
    pub kind: String,
    pub label: String,
    pub n: usize,
    pub rows: Option<usize>,
    pub cols: Option<usize>,
    pub cube_dims: Option<[usize; 3]>,
    pub layout_positions: Vec<TruePosition>,
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
    /// Chain only: m from ω² = m² + v²k² on same TFIM parameters.
    pub effective_mass: f64,
    /// Chain only: mean wavepacket group velocity from dispersion probe.
    pub dispersion_velocity_mean: f64,
    /// Chain only: mean |defect site velocity| / dispersion v_g (diagnostic; not expected ≈1).
    pub velocity_dispersion_ratio: f64,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

struct ScatterSetup {
    model: BuiltModel,
    n: usize,
    embed_dim: usize,
    defect_sites: [usize; 2],
    kind: &'static str,
    rows: Option<usize>,
    cols: Option<usize>,
    cube_dims: Option<[usize; 3]>,
}

fn resolve_scatter_setup(config: &ScatteringConfig) -> ScatterSetup {
    let kind = config.kind.as_deref().unwrap_or("chain");
    match kind {
        "grid" => {
            let rows = config.rows.unwrap_or(3).max(2);
            let cols = config.cols.unwrap_or(3).max(2);
            let model = tfim_grid(rows, cols, 1.0, config.field);
            let n = rows * cols;
            let d1 = config.defect_sites.map(|s| s[0]).unwrap_or(0);
            let d2 = config
                .defect_sites
                .map(|s| s[1])
                .unwrap_or(n.saturating_sub(1));
            ScatterSetup {
                model,
                n,
                embed_dim: 2,
                defect_sites: [d1.min(n - 1), d2.min(n - 1)],
                kind: "grid",
                rows: Some(rows),
                cols: Some(cols),
                cube_dims: None,
            }
        }
        "cube" => {
            let lx = config.lx.unwrap_or(2).max(2);
            let ly = config.ly.unwrap_or(2).max(2);
            let lz = config.lz.unwrap_or(2).max(2);
            let model = tfim_cube(lx, ly, lz, 1.0, config.field);
            let n = lx * ly * lz;
            let d1 = config.defect_sites.map(|s| s[0]).unwrap_or(0);
            let d2 = config
                .defect_sites
                .map(|s| s[1])
                .unwrap_or(n.saturating_sub(1));
            ScatterSetup {
                model,
                n,
                embed_dim: 3,
                defect_sites: [d1.min(n - 1), d2.min(n - 1)],
                kind: "cube",
                rows: None,
                cols: None,
                cube_dims: Some([lx, ly, lz]),
            }
        }
        _ => {
            let n = if config.n > 0 {
                config.n
            } else {
                12
            };
            let model = tfim_chain(n, 1.0, config.field);
            let d1 = config.defect_sites.map(|s| s[0]).unwrap_or(3);
            let d2 = config
                .defect_sites
                .map(|s| s[1])
                .unwrap_or(n.saturating_sub(4));
            ScatterSetup {
                model,
                n,
                embed_dim: 1,
                defect_sites: [d1.min(n - 1), d2.min(n - 1)],
                kind: "chain",
                rows: None,
                cols: None,
                cube_dims: None,
            }
        }
    }
}

fn manhattan_distance(a: &TruePosition, b: &TruePosition) -> f64 {
    let mut d = (a.x - b.x).abs() + (a.y - b.y).abs();
    match (a.z, b.z) {
        (Some(za), Some(zb)) => d += (za - zb).abs(),
        _ => {}
    }
    d
}

pub(crate) fn lattice_separation(site_a: usize, site_b: usize, positions: &[TruePosition]) -> f64 {
    let a = site_a.min(positions.len().saturating_sub(1));
    let b = site_b.min(positions.len().saturating_sub(1));
    manhattan_distance(&positions[a], &positions[b])
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

fn fit_graph_distance_velocity(
    worldline: &[ScatteringWorldlinePoint],
    initial: usize,
    positions: &[TruePosition],
    burn_in_frac: f64,
) -> f64 {
    let start = ((worldline.len() as f64) * burn_in_frac).floor() as usize;
    if worldline.len().saturating_sub(start) < 2 {
        return 0.0;
    }
    let mut sum_t = 0.0;
    let mut sum_d = 0.0;
    let mut sum_tt = 0.0;
    let mut sum_td = 0.0;
    let mut count = 0.0;
    for p in worldline.iter().skip(start) {
        let t = p.t;
        let d = lattice_separation(p.site, initial, positions);
        sum_t += t;
        sum_d += d;
        sum_tt += t * t;
        sum_td += t * d;
        count += 1.0;
    }
    let denom = count * sum_tt - sum_t * sum_t;
    if denom.abs() < 1e-12 {
        return 0.0;
    }
    (count * sum_td - sum_t * sum_d) / denom
}

fn worldline_moved(
    worldline: &[ScatteringWorldlinePoint],
    initial: usize,
    positions: Option<&[TruePosition]>,
) -> bool {
    if let Some(pos) = positions {
        worldline
            .iter()
            .any(|p| lattice_separation(p.site, initial, pos) >= 1.0)
    } else {
        worldline
            .iter()
            .any(|p| (p.site as i32 - initial as i32).abs() >= 1)
    }
}

fn analyze_crossing(
    wl1: &[ScatteringWorldlinePoint],
    wl2: &[ScatteringWorldlinePoint],
    d1: usize,
    d2: usize,
    positions: Option<&[TruePosition]>,
) -> (bool, f64, Vec<f64>) {
    let mut min_sep = f64::INFINITY;
    let mut crossed = false;
    let mut separation_series = Vec::with_capacity(wl1.len());
    let overlap_start = wl1.len() * 15 / 100;
    for (k, (l, r)) in wl1.iter().zip(wl2.iter()).enumerate() {
        let sep = if let Some(pos) = positions {
            lattice_separation(l.site, r.site, pos)
        } else {
            (l.site as i32 - r.site as i32).unsigned_abs() as f64
        };
        separation_series.push(sep);
        if k >= overlap_start {
            min_sep = min_sep.min(sep);
            if let Some(pos) = positions {
                let x0_d1 = pos[d1.min(pos.len() - 1)].x;
                let x0_d2 = pos[d2.min(pos.len() - 1)].x;
                let x_l = pos[l.site.min(pos.len() - 1)].x;
                let x_r = pos[r.site.min(pos.len() - 1)].x;
                if x0_d1 < x0_d2 && x_l >= x_r {
                    crossed = true;
                }
                if x0_d1 > x0_d2 && x_l <= x_r {
                    crossed = true;
                }
            } else if d1 < d2 && l.site >= r.site {
                crossed = true;
            } else if d1 > d2 && l.site <= r.site {
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
    hamiltonian: &Hamiltonian,
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

fn two_defect_initial(n: usize, d1: usize, d2: usize) -> QuantumState {
    let mut initial = QuantumState::zero(n);
    initial.data[2 * (1 << d1)] = 1.0;
    initial.data[2 * (1 << d2)] = 1.0;
    initial
}

pub fn run_two_defect_scattering(config: &ScatteringConfig) -> ScatteringResult {
    let setup = resolve_scatter_setup(config);
    let d1 = setup.defect_sites[0];
    let d2 = setup.defect_sites[1];
    let positions = setup.model.layout.true_positions.clone();
    let pos_ref = if setup.embed_dim >= 2 {
        Some(positions.as_slice())
    } else {
        None
    };

    let initial = two_defect_initial(setup.n, d1, d2);
    let reference = {
        let mut r = QuantumState::zero(setup.n);
        r.data[0] = 1.0;
        r
    };

    let lite = config.lite;
    let initial_for_phase = initial.clone_state();
    let reference_for_phase = reference.clone_state();
    let spacetime = build_light_cone_trajectory(SpacetimeConfig {
        hamiltonian: &setup.model.hamiltonian,
        initial,
        reference: Some(reference),
        dt: config.dt,
        steps: config.steps,
        embed_dim: setup.embed_dim,
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
    let (crossed, min_separation, separation_series) =
        analyze_crossing(&wl1, &wl2, d1, d2, pos_ref);
    let velocities = if setup.embed_dim >= 2 {
        [
            fit_graph_distance_velocity(&wl1, d1, &positions, 0.15),
            fit_graph_distance_velocity(&wl2, d2, &positions, 0.15),
        ]
    } else {
        [
            fit_site_velocity(&wl1, 0.15),
            fit_site_velocity(&wl2, 0.15),
        ]
    };
    let both_moved =
        worldline_moved(&wl1, d1, pos_ref) && worldline_moved(&wl2, d2, pos_ref);
    let (phase_series, post_interaction_phase_std, phase_stable) = track_exchange_phases(
        &setup.model.hamiltonian,
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

    let (effective_mass, dispersion_velocity_mean, velocity_dispersion_ratio) =
        if setup.kind == "chain" {
            let disp = build_dispersion(
                &setup.model.hamiltonian,
                &DispersionConfig {
                    n: setup.n,
                    field: config.field,
                    dt: config.dt,
                    steps: config.steps,
                    modes: 3,
                },
            );
            let v_mean = disp.group_velocity_mean;
            let v_scatter = (velocities[0].abs() + velocities[1].abs()) / 2.0;
            let ratio = if v_mean > 1e-9 {
                v_scatter / v_mean
            } else {
                f64::NAN
            };
            (disp.effective_mass, v_mean, ratio)
        } else {
            (f64::NAN, f64::NAN, f64::NAN)
        };

    ScatteringResult {
        kind: setup.kind.to_string(),
        label: setup.model.label.clone(),
        n: setup.n,
        rows: setup.rows,
        cols: setup.cols,
        cube_dims: setup.cube_dims,
        layout_positions: positions,
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
        effective_mass,
        dispersion_velocity_mean,
        velocity_dispersion_ratio,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_chain_config() -> ScatteringConfig {
        ScatteringConfig {
            n: 12,
            field: 0.7,
            dt: 0.12,
            steps: 40,
            defect_sites: Some([3, 8]),
            kind: None,
            rows: None,
            cols: None,
            lx: None,
            ly: None,
            lz: None,
            lite: true,
            taylor_order: 4,
        }
    }

    fn default_grid_config() -> ScatteringConfig {
        ScatteringConfig {
            n: 9,
            field: 0.7,
            dt: 0.12,
            steps: 40,
            defect_sites: Some([0, 8]),
            kind: Some("grid".to_string()),
            rows: Some(3),
            cols: Some(3),
            lx: None,
            ly: None,
            lz: None,
            lite: true,
            taylor_order: 4,
        }
    }

    #[test]
    fn interaction_metrics_on_default_config() {
        let r = run_two_defect_scattering(&default_chain_config());
        assert!(r.overlap_detected, "cones should close on default demo");
        assert!(
            r.overlap_step > 0 && r.overlap_step < r.separation_series.len(),
            "overlap_step={} out of range",
            r.overlap_step
        );
        assert!(r.interaction_phase_shift.is_finite());
        assert!(r.separation_time_delay.is_finite());
        assert_eq!(r.kind, "chain");
        assert!(r.effective_mass.is_finite() && r.effective_mass > 0.0);
        assert!(r.velocity_dispersion_ratio.is_finite() && r.velocity_dispersion_ratio > 0.5);
    }

    #[test]
    fn interaction_phase_shift_exceeds_ad_threshold() {
        let r = run_two_defect_scattering(&default_chain_config());
        assert!(
            r.interaction_phase_shift.abs() > 0.05,
            "AD threshold: |phase_shift|={:.4}",
            r.interaction_phase_shift
        );
    }

    #[test]
    fn grid_scattering_both_defects_propagate() {
        let r = run_two_defect_scattering(&default_grid_config());
        assert_eq!(r.kind, "grid");
        assert_eq!(r.rows, Some(3));
        assert_eq!(r.cols, Some(3));
        assert!(r.both_moved, "both defects should move on 3x3 grid");
        let init_sep = lattice_separation(
            r.defect_sites[0],
            r.defect_sites[1],
            &r.layout_positions,
        );
        assert!(
            r.min_separation < init_sep,
            "graph separation should decrease from initial {}, got min {}",
            init_sep,
            r.min_separation
        );
        assert_eq!(r.layout_positions.len(), 9);
    }

    #[test]
    fn cube_scattering_runs() {
        let r = run_two_defect_scattering(&ScatteringConfig {
            n: 8,
            field: 0.7,
            dt: 0.12,
            steps: 32,
            defect_sites: Some([0, 7]),
            kind: Some("cube".to_string()),
            rows: None,
            cols: None,
            lx: Some(2),
            ly: Some(2),
            lz: Some(2),
            lite: true,
            taylor_order: 4,
        });
        assert_eq!(r.kind, "cube");
        assert_eq!(r.cube_dims, Some([2, 2, 2]));
        assert!(r.both_moved);
    }

    #[test]
    fn effective_mass_af_criteria_on_chain() {
        let s = run_two_defect_scattering(&default_chain_config());
        assert!(s.effective_mass > 0.01);
        assert!(s.dispersion_velocity_mean > 0.01);
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
