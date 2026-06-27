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
pub(crate) fn blind_locality_fraction(h: &Hamiltonian, ground: &QuantumState) -> f64 {
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

/// Like `evaluate_point` but skips the expensive blind-locality factorization search.
/// Used for fast unit tests and the field-sweep where locality is secondary.
fn evaluate_point_fast(n: usize, field: f64) -> HolographicBoundPoint {
    let model = tfim_chain(n, 1.0, field);
    let mut rng = Rng::new(42 + n as u32);
    let (ground, _, _) = ground_state(&model.hamiltonian, &mut rng, 4000, 1e-9);
    let random = make_random_state(n, &mut Rng::new(99));
    let holo = analyze_holography(&ground, &random);
    let dim = analyze_emergent_geometry(&ground, 1.0, 3).mds.emergent_dim;
    let area_ok = area_law_ok(&ground, holo.rt_r2);
    let dim_ok = dim <= 2;
    HolographicBoundPoint {
        n,
        rt_r2: holo.rt_r2,
        rt_slope: holo.rt_slope,
        emergent_dim: dim,
        locality_fraction: f64::NAN, // not computed
        area_law_ok: area_ok,
        dim_ok,
        locality_ok: false, // not computed
        all_ok: area_ok && dim_ok, // locality excluded
    }
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

/// One point in the field sweep: the n_min at a given transverse-field value.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldSweepPoint {
    pub field: f64,
    /// Holographic bound scan result at this field value.
    pub scan: HolographicBoundResult,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldSweepResult {
    pub points: Vec<FieldSweepPoint>,
    /// Field value at which n_min is largest (None if no field passes diagnostics).
    pub field_max_n_min: Option<f64>,
    /// The largest n_min observed across the sweep.
    pub n_min_max: Option<usize>,
    /// Field value at which n_min first becomes non-None (holographic emergence transition).
    pub h_holographic_emergence: Option<f64>,
    /// True if the holographic emergence transition falls near the TFIM critical point
    /// (0.7 ≤ h_transition ≤ 1.3): ordered phase lacks RT structure, disordered phase has it.
    pub emergence_near_critical: bool,
    /// True if every sub-critical point (h < h_transition) has n_min = None.
    pub ordered_phase_nonholographic: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldSweepConfig {
    /// Field values to sweep (ordered low → high).
    /// Defaults to [0.3, 0.5, 0.7, 1.0, 1.2, 1.5] spanning ordered→critical→disordered.
    #[serde(default = "default_sweep_fields")]
    pub fields: Vec<f64>,
    #[serde(default = "default_sweep_n_min")]
    pub n_min: usize,
    #[serde(default = "default_sweep_n_max")]
    pub n_max: usize,
    /// Skip blind-locality factorization search (much faster; suitable for initial sweep).
    #[serde(default)]
    pub skip_locality: bool,
}

fn default_sweep_fields() -> Vec<f64> {
    vec![0.3, 0.5, 0.7, 1.0, 1.2, 1.5]
}
fn default_sweep_n_min() -> usize { 6 }
fn default_sweep_n_max() -> usize { 12 }

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldSweepRunResult {
    #[serde(flatten)]
    pub inner: FieldSweepResult,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

pub fn run_field_sweep(config: &FieldSweepConfig) -> FieldSweepResult {
    let points: Vec<FieldSweepPoint> = config.fields.iter().map(|&h| {
        let mut scan_points = Vec::new();
        for n in config.n_min..=config.n_max {
            scan_points.push(if config.skip_locality {
                evaluate_point_fast(n, h)
            } else {
                evaluate_point(n, h)
            });
        }
        let n_bound = scan_points.iter().find(|p| p.all_ok).map(|p| p.n);
        let n_saturation = scan_points.iter().filter(|p| p.all_ok).count();
        let bound_scales = if let (Some(first), Some(last)) = (
            scan_points.iter().find(|p| p.all_ok).map(|p| p.n),
            scan_points.last().filter(|p| p.all_ok).map(|p| p.n),
        ) {
            last >= first && n_saturation >= 2
        } else {
            false
        };
        FieldSweepPoint {
            field: h,
            scan: HolographicBoundResult { field: h, points: scan_points, n_min: n_bound, n_saturation, bound_scales },
        }
    }).collect();

    let field_max_n_min = points.iter()
        .filter_map(|p| p.scan.n_min.map(|n| (p.field, n)))
        .max_by_key(|&(_, n)| n)
        .map(|(h, _)| h);
    let n_min_max = points.iter().filter_map(|p| p.scan.n_min).max();

    // Holographic emergence: the first field value where n_min becomes non-None.
    let h_holographic_emergence = points.iter()
        .find(|p| p.scan.n_min.is_some())
        .map(|p| p.field);

    // True if that transition falls in the range [0.7, 1.3] — near the TFIM critical point.
    let emergence_near_critical = h_holographic_emergence
        .map(|h| h >= 0.7 && h <= 1.3)
        .unwrap_or(false);

    // True if all fields strictly below h_transition have n_min = None.
    let ordered_phase_nonholographic = h_holographic_emergence.map(|h_t| {
        points.iter()
            .filter(|p| p.field < h_t)
            .all(|p| p.scan.n_min.is_none())
    }).unwrap_or(false);

    FieldSweepResult {
        points,
        field_max_n_min,
        n_min_max,
        h_holographic_emergence,
        emergence_near_critical,
        ordered_phase_nonholographic,
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

pub fn run_field_sweep_probe(config: &FieldSweepConfig) -> FieldSweepRunResult {
    FieldSweepRunResult {
        inner: run_field_sweep(config),
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

    /// Fast field sweep diagnostic — prints n_min at each field with skip_locality.
    #[test]
    fn field_sweep_print_diagnostic() {
        let r = run_field_sweep(&FieldSweepConfig {
            fields: vec![0.3, 0.5, 0.7, 1.0, 1.2, 1.5],
            n_min: 6,
            n_max: 10,
            skip_locality: true,
        });
        for p in &r.points {
            let n_min_str = p.scan.n_min.map(|n| n.to_string()).unwrap_or_else(|| "None".to_string());
            println!("h={:.1}  n_min={}  field_max={:?}  peaks_near_critical={}",
                p.field, n_min_str, r.field_max_n_min, r.emergence_near_critical);
        }
    }

    /// Field sweep: at least one field value should pass holographic diagnostics.
    #[test]
    fn field_sweep_n_min_varies_across_phases() {
        let r = run_field_sweep(&FieldSweepConfig {
            fields: vec![0.3, 0.7, 1.0, 1.5],
            n_min: 6,
            n_max: 8,
            skip_locality: true,
        });
        let any_pass = r.points.iter().any(|p| p.scan.n_min.is_some());
        assert!(any_pass, "at least one field value should find a holographic n_min");
    }

    /// Falsification AC: holographic RT structure is absent in the ordered phase (h<1)
    /// and emerges near the TFIM critical point (h≈1.0).
    /// The ordered-phase ground state is a cat state — S(A)≈const for all bipartitions,
    /// R²≈0 in the RT fit — so n_min=None. Above criticality, area-law entanglement
    /// gives a clean RT fit and finite n_min.
    #[test]
    fn holographic_structure_emerges_at_critical_point() {
        let r = run_field_sweep(&FieldSweepConfig {
            fields: vec![0.3, 0.7, 1.0, 1.5],
            n_min: 6,
            n_max: 8,
            skip_locality: true,
        });
        assert!(
            r.emergence_near_critical,
            "holographic emergence transition should fall near h_critical≈1.0; \
             h_transition={:?}",
            r.h_holographic_emergence,
        );
        assert!(
            r.ordered_phase_nonholographic,
            "all fields below the transition should have n_min=None; \
             h_transition={:?}",
            r.h_holographic_emergence,
        );
    }
}
