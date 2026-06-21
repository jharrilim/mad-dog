//! Two-defect scattering — light-cone worldline tracking.

use crate::models::tfim_chain;
use crate::quantum::QuantumState;
use crate::spacetime::{build_light_cone_trajectory, SpacetimeConfig, SpacetimeSlice};
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
pub struct WorldlinePoint {
    pub t: f64,
    pub site: usize,
    pub amplitude: f64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScatteringResult {
    pub n: usize,
    pub field: f64,
    pub defect_sites: [usize; 2],
    pub slices: Vec<SpacetimeSlice>,
    pub worldlines: [Vec<WorldlinePoint>; 2],
    pub crossed: bool,
    pub min_separation: f64,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

fn track_worldline(
    slices: &[SpacetimeSlice],
    defect_site: usize,
    n: usize,
    side: &str,
) -> Vec<WorldlinePoint> {
    let mid = n / 2;
    slices
        .iter()
        .map(|slice| {
            let (lo, hi) = if side == "left" {
                (0usize, mid.saturating_sub(1))
            } else {
                (mid, n - 1)
            };
            let mut best_site = defect_site;
            let mut best_amp = -1.0;
            for i in lo..=hi {
                if slice.signal[i] > best_amp {
                    best_amp = slice.signal[i];
                    best_site = i;
                }
            }
            WorldlinePoint {
                t: slice.t,
                site: best_site,
                amplitude: best_amp,
            }
        })
        .collect()
}

fn analyze_crossing(
    left: &[WorldlinePoint],
    right: &[WorldlinePoint],
    d1: usize,
    d2: usize,
) -> (bool, f64) {
    let mut min_sep = f64::INFINITY;
    let mut crossed = false;
    let overlap_start = left.len() * 15 / 100;
    for k in overlap_start..left.len() {
        let sep = (left[k].site as i32 - right[k].site as i32).unsigned_abs() as f64;
        min_sep = min_sep.min(sep);
        if d1 < d2 && left[k].site >= right[k].site {
            crossed = true;
        }
        if d1 > d2 && left[k].site <= right[k].site {
            crossed = true;
        }
    }
    (
        crossed,
        if min_sep.is_finite() { min_sep } else { 0.0 },
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
    });

    let wl1 = track_worldline(&spacetime.slices, d1, config.n, "left");
    let wl2 = track_worldline(&spacetime.slices, d2, config.n, "right");
    let (crossed, min_separation) = analyze_crossing(&wl1, &wl2, d1, d2);

    ScatteringResult {
        n: config.n,
        field: config.field,
        defect_sites: [d1, d2],
        slices: if config.lite {
            Vec::new()
        } else {
            spacetime.slices
        },
        worldlines: [wl1, wl2],
        crossed,
        min_separation,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}
