//! RT diagnostics time series under defect quench (Phase 5 / S5).

use crate::geometry::{entropy_of_region, mutual_information_matrix};
use crate::geometry_stability::spearman_corr;
use crate::holography::analyze_rt_relation;
use crate::quantum::QuantumState;
use serde::{Deserialize, Serialize};

/// Mean |S_A / (½ boundary MI) − 1| over all contiguous intervals.
pub fn mean_rt_ratio_deviation(state: &QuantumState) -> f64 {
    let n = state.n;
    let mi = mutual_information_matrix(state);
    let mut devs = Vec::new();
    for start in 0..n {
        for len in 1..=(n - start) {
            let region: Vec<usize> = (start..start + len).collect();
            let s_a = entropy_of_region(state, &region);
            let end = start + len - 1;
            let mut boundary_mi = 0.0;
            for a in start..=end {
                for b in 0..n {
                    if b < start || b > end {
                        boundary_mi += mi[a][b];
                    }
                }
            }
            let cut = 0.5 * boundary_mi;
            if cut > 1e-9 {
                devs.push((s_a / cut - 1.0).abs());
            }
        }
    }
    if devs.is_empty() {
        return 0.0;
    }
    devs.iter().sum::<f64>() / devs.len() as f64
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RtQuenchSlice {
    pub t: f64,
    pub step: usize,
    pub rt_slope: f64,
    pub rt_r2: f64,
    pub mean_rt_ratio_dev: f64,
    pub slope_deviation: f64,
    pub area_pressure: f64,
}

/// Spearman ρ between area pressure and RT slope deviation across quench slices.
pub fn rt_deviation_density_correlation(slices: &[RtQuenchSlice]) -> f64 {
    if slices.len() < 3 {
        return 0.0;
    }
    let density: Vec<f64> = slices.iter().map(|s| s.area_pressure).collect();
    let slope_dev: Vec<f64> = slices.iter().map(|s| s.slope_deviation).collect();
    spearman_corr(&density, &slope_dev)
}

/// Structured deviation: RT slope deficit tracks excitation density proxy.
pub fn structured_rt_deviation(slices: &[RtQuenchSlice], min_corr: f64) -> bool {
    if slices.len() < 4 {
        return false;
    }
    let corr = rt_deviation_density_correlation(slices);
    if corr.abs() >= min_corr {
        return true;
    }
    let early = slices.get(2).map(|s| s.slope_deviation).unwrap_or(0.0);
    let late = slices.last().map(|s| s.slope_deviation).unwrap_or(0.0);
    late > early + 0.08
}

pub fn rt_quench_slice_from_state(
    state: &QuantumState,
    step: usize,
    t: f64,
    area_pressure: f64,
    max_slope_dev: f64,
) -> RtQuenchSlice {
    let RtFit { rt_slope, rt_r2, .. } = analyze_rt_relation(state, None);
    let rt_slope_safe = if rt_slope.is_finite() { rt_slope } else { 1.0 };
    let rt_r2_safe = if rt_r2.is_finite() { rt_r2 } else { 0.0 };
    let slope_deviation = ((rt_slope_safe - 1.0).abs() / max_slope_dev).clamp(0.0, 1.0);
    RtQuenchSlice {
        t,
        step,
        rt_slope: rt_slope_safe,
        rt_r2: rt_r2_safe,
        mean_rt_ratio_dev: mean_rt_ratio_deviation(state),
        slope_deviation,
        area_pressure,
    }
}

use crate::holography::RtFit;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::tfim_chain;
    use crate::quantum::{evolve_interval, ground_state};
    use crate::refinement::{
        compute_refinement_baselines, defect_initial_state, measure_refinement_diagnostics,
        RefinementThresholds,
    };
    use crate::rng::Rng;

    #[test]
    fn ground_rt_ratio_dev_small() {
        let model = tfim_chain(10, 1.0, 1.5);
        let (ground, _, _) = ground_state(&model.hamiltonian, &mut Rng::new(42), 4000, 1e-9);
        let dev = mean_rt_ratio_deviation(&ground);
        assert!(dev < 0.2, "ground mean RT ratio dev={dev}");
    }

    #[test]
    fn quench_slices_show_structured_deviation() {
        let n = 10;
        let field = 1.5;
        let dt = 0.2;
        let model = tfim_chain(n, 1.0, field);
        let mut rng = Rng::new(42);
        let radius = model
            .hamiltonian
            .estimate_spectral_radius(&mut rng, 30)
            .max(1e-6);
        let baselines = compute_refinement_baselines(n, field, 7711);
        let thresholds = RefinementThresholds::default();
        let mut state = defect_initial_state(n);
        let mut slices = Vec::new();
        for step in 0..=16 {
            let diag = measure_refinement_diagnostics(&state, 1, &baselines, &thresholds);
            slices.push(rt_quench_slice_from_state(
                &state,
                step,
                step as f64 * dt,
                diag.area_pressure,
                thresholds.max_rt_slope_dev,
            ));
            if step < 16 {
                state = evolve_interval(&model.hamiltonian, &state, dt, radius, 6);
            }
        }
        assert!(
            structured_rt_deviation(&slices, 0.3),
            "expected structured RT deviation under quench"
        );
    }
}
