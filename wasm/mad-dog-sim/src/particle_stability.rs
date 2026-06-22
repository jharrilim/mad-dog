//! Defect localization lifetime vs transverse field (Phase 6 / S6).

use crate::models::tfim_chain;
use crate::quantum::QuantumState;
use crate::spacetime::{build_light_cone_trajectory, SpacetimeConfig};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleLifetimeCase {
    pub field: f64,
    pub localization_fraction: f64,
    pub mean_peak_width: f64,
    pub worldline_jitter: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleStabilityResult {
    pub n: usize,
    pub dt: f64,
    pub steps: usize,
    pub ordered: ParticleLifetimeCase,
    pub disordered: ParticleLifetimeCase,
    pub ordered_longer_lived: bool,
}

fn peak_width(signal: &[f64]) -> f64 {
    let max = signal.iter().copied().fold(0.0_f64, f64::max);
    if max < 1e-12 {
        return signal.len() as f64;
    }
    let thresh = 0.5 * max;
    let active: Vec<usize> = signal
        .iter()
        .enumerate()
        .filter(|(_, v)| **v >= thresh)
        .map(|(i, _)| i)
        .collect();
    if active.is_empty() {
        return signal.len() as f64;
    }
    (*active.iter().max().unwrap() - active.iter().min().unwrap() + 1) as f64
}

fn run_lifetime_case(n: usize, field: f64, dt: f64, steps: usize) -> ParticleLifetimeCase {
    let model = tfim_chain(n, 1.0, field);
    let center = n / 2;
    let mut initial = QuantumState::zero(n);
    initial.data[2 * (1 << center)] = 1.0;
    let result = build_light_cone_trajectory(SpacetimeConfig {
        hamiltonian: &model.hamiltonian,
        initial,
        reference: None,
        dt,
        steps,
        embed_dim: 1,
        align_to: None,
        order: 4,
        include_geometry: false,
        track_energy: false,
        retain_slices: true,
        worldline_defects: None,
        track_worldline: true,
        defect_site: Some(center),
        signal_reference_site: None,
        seed: 42,
    });

    let radius = 2usize;
    let worldline = result.worldline.unwrap_or_default();
    let localized = worldline
        .iter()
        .filter(|p| p.site.abs_diff(center) <= radius)
        .count();
    let localization_fraction = localized as f64 / worldline.len().max(1) as f64;

    let widths: Vec<f64> = result
        .slices
        .iter()
        .map(|s| peak_width(&s.signal))
        .collect();
    let mean_peak_width = if widths.is_empty() {
        0.0
    } else {
        widths.iter().sum::<f64>() / widths.len() as f64
    };

    let mut jitter = 0.0;
    for w in worldline.windows(2) {
        jitter += (w[1].site as f64 - w[0].site as f64).abs();
    }
    let worldline_jitter = if worldline.len() > 1 {
        jitter / (worldline.len() - 1) as f64
    } else {
        0.0
    };

    ParticleLifetimeCase {
        field,
        localization_fraction,
        mean_peak_width,
        worldline_jitter,
    }
}

pub fn run_particle_stability(n: usize, dt: f64, steps: usize) -> ParticleStabilityResult {
    let ordered = run_lifetime_case(n, 0.5, dt, steps);
    let disordered = run_lifetime_case(n, 2.5, dt, steps);
    let ordered_longer_lived = ordered.localization_fraction > disordered.localization_fraction + 0.05
        || ordered.mean_peak_width < disordered.mean_peak_width - 0.3;

    ParticleStabilityResult {
        n,
        dt,
        steps,
        ordered,
        disordered,
        ordered_longer_lived,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordered_phase_defect_more_localized() {
        let r = run_particle_stability(10, 0.2, 24);
        assert!(
            r.ordered_longer_lived,
            "ordered={:.3} disordered={:.3}",
            r.ordered.localization_fraction,
            r.disordered.localization_fraction
        );
    }
}
