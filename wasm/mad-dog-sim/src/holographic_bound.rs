//! Minimal n for holographic signatures on TFIM chains (Phase 7 / S10).
//!
//! Locality is measured via blind factorization on |ψ⟩ (MI annealing), not native Ĥ line
//! locality — the latter is tautological on TFIM chains (Phase 9 / W′ hardening).

use crate::factorization::{search_factorization, SearchMethod, SearchParams};
use crate::geometry::analyze_emergent_geometry;
use crate::holography::analyze_holography;
use crate::models::tfim_chain;
use crate::quantum::{ground_state, make_random_state, Hamiltonian, QuantumState};
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

/// Annealing budget for blind locality on n > 8 (exact enumeration is used for n ≤ 8).
fn blind_annealing_steps(n: usize) -> usize {
    600 + n * 80
}

/// Locality fraction from permutation search on |ψ⟩ (Pauli + MI scorer).
fn blind_locality_fraction(h: &Hamiltonian, ground: &QuantumState) -> f64 {
    let n = ground.n;
    let params = SearchParams {
        search_method: if n <= 8 {
            SearchMethod::Exact
        } else {
            SearchMethod::Annealing
        },
        annealing_steps: blind_annealing_steps(n),
        eigenstate_count: 1,
        ..Default::default()
    };
    search_factorization(h, std::slice::from_ref(ground), 1, &params, None)
        .best
        .locality_fraction
}

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
    let locality = blind_locality_fraction(&model.hamiltonian, &ground);
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
    use crate::factorization::shuffle_hamiltonian;
    use crate::models::line_locality_fraction;

    #[test]
    fn finds_finite_n_min_for_ordered_chain() {
        let r = run_holographic_bound_scan(1.5, 6, 12);
        assert!(r.n_min.is_some(), "expected holographic bound n_min");
        assert!(r.bound_scales);
    }

    #[test]
    fn blind_locality_high_on_native_chain_ground() {
        let n = 8;
        let model = tfim_chain(n, 1.0, 1.5);
        let mut rng = Rng::new(77);
        let (ground, _, _) = ground_state(&model.hamiltonian, &mut rng, 4000, 1e-9);
        let blind = blind_locality_fraction(&model.hamiltonian, &ground);
        assert!(
            blind >= 0.95,
            "native chain ground state should recover line locality, got {blind:.3}"
        );
    }

    #[test]
    fn shuffled_chain_fails_native_locality_leg() {
        let n = 8;
        let model = tfim_chain(n, 1.0, 1.5);
        let (shuffled, _) = shuffle_hamiltonian(&model.hamiltonian, 4242);
        let native_locality = line_locality_fraction(&shuffled);
        assert!(
            native_locality < 0.95,
            "shuffled labels should fail tautological native-line locality, got {native_locality:.3}"
        );
        let mut rng = Rng::new(77);
        let (ground, _, _) = ground_state(&shuffled, &mut rng, 4000, 1e-9);
        let blind = blind_locality_fraction(&shuffled, &ground);
        assert!(
            blind >= 0.95,
            "blind factorization should recover locality on shuffled chain, got {blind:.3}"
        );
    }

    #[test]
    fn n_min_stable_on_hold_out_ground_seeds() {
        let field = 1.5;
        let baseline = run_holographic_bound_scan(field, 6, 10);
        assert!(baseline.n_min.is_some(), "baseline seed grid should find n_min");

        let mut hold_out_points = Vec::new();
        for n in 6..=10 {
            let model = tfim_chain(n, 1.0, field);
            let mut rng = Rng::new(9000 + n as u32);
            let (ground, _, _) = ground_state(&model.hamiltonian, &mut rng, 4000, 1e-9);
            let random = make_random_state(n, &mut Rng::new(99));
            let holo = analyze_holography(&ground, &random);
            let dim = analyze_emergent_geometry(&ground, 1.0, 3).mds.emergent_dim;
            let locality = blind_locality_fraction(&model.hamiltonian, &ground);
            let area_ok = area_law_ok(&ground, holo.rt_r2);
            let dim_ok = dim <= 2;
            let locality_ok = locality >= 0.95;
            hold_out_points.push(area_ok && dim_ok && locality_ok);
        }
        let hold_out_n_min = hold_out_points
            .iter()
            .enumerate()
            .find(|(_, ok)| **ok)
            .map(|(i, _)| 6 + i);
        assert_eq!(
            baseline.n_min, hold_out_n_min,
            "n_min should match between default and hold-out ground-state seeds"
        );
    }
}
