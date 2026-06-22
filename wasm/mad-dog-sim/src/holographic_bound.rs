//! Minimal n for holographic signatures on TFIM chains (Phase 7 / S10).

use crate::geometry::analyze_emergent_geometry;
use crate::holography::analyze_holography;
use crate::models::{line_locality_fraction, tfim_chain};
use crate::quantum::{ground_state, make_random_state, QuantumState};
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HolographicBoundPoint {
    pub n: usize,
    pub rt_r2: f64,
    pub rt_slope: f64,
    pub emergent_dim: usize,
    pub locality_fraction: f64,
    pub area_law_ok: bool,
    pub dim_ok: bool,
    pub locality_ok: bool,
    pub all_ok: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HolographicBoundResult {
    pub field: f64,
    pub points: Vec<HolographicBoundPoint>,
    pub n_min: Option<usize>,
    pub n_saturation: usize,
    pub bound_scales: bool,
}

fn area_law_ok(ground: &QuantumState, holo_rt_r2: f64) -> bool {
    let n = ground.n;
    if n < 4 || holo_rt_r2 < 0.65 {
        return false;
    }
    let s1 = crate::geometry::entropy_of_region(ground, &(0..1).collect::<Vec<_>>());
    let smid = crate::geometry::entropy_of_region(ground, &(0..(n / 2).max(1)).collect::<Vec<_>>());
    smid > s1 * 0.5
}

fn evaluate_point(n: usize, field: f64) -> HolographicBoundPoint {
    let model = tfim_chain(n, 1.0, field);
    let mut rng = Rng::new(42 + n as u32);
    let (ground, _, _) = ground_state(&model.hamiltonian, &mut rng, 4000, 1e-9);
    let random = make_random_state(n, &mut Rng::new(99));
    let holo = analyze_holography(&ground, &random);
    let dim = analyze_emergent_geometry(&ground, 1.0, 3).mds.emergent_dim;
    let locality = line_locality_fraction(&model.hamiltonian);
    let area_ok = area_law_ok(&ground, holo.rt_r2);
    let dim_ok = dim <= 2;
    let locality_ok = locality >= 0.95;
    HolographicBoundPoint {
        n,
        rt_r2: holo.rt_r2,
        rt_slope: holo.rt_slope,
        emergent_dim: dim,
        locality_fraction: locality,
        area_law_ok: area_ok,
        dim_ok,
        locality_ok,
        all_ok: area_ok && dim_ok && locality_ok,
    }
}

pub fn run_holographic_bound_scan(field: f64, n_min: usize, n_max: usize) -> HolographicBoundResult {
    let mut points = Vec::new();
    for n in n_min..=n_max {
        points.push(evaluate_point(n, field));
    }
    let n_bound = points.iter().find(|p| p.all_ok).map(|p| p.n);
    let n_saturation = points.iter().filter(|p| p.all_ok).count();
    let bound_scales = if let (Some(first), Some(last)) = (
        points.iter().find(|p| p.all_ok).map(|p| p.n),
        points.last().filter(|p| p.all_ok).map(|p| p.n),
    ) {
        last >= first && n_saturation >= 2
    } else {
        false
    };

    HolographicBoundResult {
        field,
        points,
        n_min: n_bound,
        n_saturation,
        bound_scales,
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HolographicBoundConfig {
    pub field: f64,
    #[serde(default = "default_n_min")]
    pub n_min: usize,
    #[serde(default = "default_n_max")]
    pub n_max: usize,
}

fn default_n_min() -> usize {
    6
}

fn default_n_max() -> usize {
    12
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HolographicBoundRunResult {
    #[serde(flatten)]
    pub inner: HolographicBoundResult,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

pub fn run_holographic_bound(config: &HolographicBoundConfig) -> HolographicBoundRunResult {
    HolographicBoundRunResult {
        inner: run_holographic_bound_scan(config.field, config.n_min, config.n_max),
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_finite_n_min_for_ordered_chain() {
        let r = run_holographic_bound_scan(1.5, 6, 12);
        assert!(r.n_min.is_some(), "expected holographic bound n_min");
        assert!(r.bound_scales);
    }
}
