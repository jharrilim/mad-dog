//! Two-defect scattering — light-cone worldline tracking.

use crate::models::tfim_chain;
use crate::quantum::QuantumState;
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
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}
