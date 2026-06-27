//! N-site modular-flow clock networks — entropy-threshold clocks on lattices.

use crate::modular_time::{cumulative_modular_time, event_ticks_from_cumulative};
use crate::multi_clock::{
    default_clock_sites_chain, default_clock_sites_cube, default_clock_sites_grid,
};
use crate::quantum::QuantumState;
use crate::relational_time::{
    evolve_trajectory, fit_affine, physical_time_at_uniform, TimeMapPoint,
};
use serde::{Deserialize, Serialize};

fn default_modular_slices() -> usize {
    12
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModularMultiClockConfig {
    pub n: usize,
    pub dt: f64,
    pub steps: usize,
    #[serde(default)]
    pub clock_sites: Option<Vec<usize>>,
    #[serde(default = "default_modular_slices")]
    pub modular_slices: usize,
    #[serde(default)]
    pub defect_site: Option<usize>,
    #[serde(default)]
    pub edge_sites: Option<[usize; 2]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModularClockReading {
    pub site: usize,
    pub label: String,
    pub modular_tau: Vec<f64>,
    pub time_map: Vec<TimeMapPoint>,
    pub sync_r2_vs_uniform: f64,
    pub sync_slope_vs_uniform: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModularMultiClockResult {
    pub kind: String,
    pub label: String,
    pub sites: usize,
    pub defect_site: usize,
    pub labels: Vec<String>,
    pub clocks: Vec<ModularClockReading>,
    pub pairwise_r2: Vec<Vec<f64>>,
    pub min_pairwise_r2: f64,
    pub defect_uniform_r2: f64,
    pub edge_edge_r2: f64,
    pub inconsistent_pairs: usize,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

fn uniform_times(steps: usize) -> Vec<f64> {
    (0..steps).map(|k| k as f64).collect()
}

fn pairwise_sync_r2(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len());
    if n < 2 {
        return 1.0;
    }
    fit_affine(&a[..n], &b[..n]).2
}

fn sync_r2_time_map(time_map: &[TimeMapPoint]) -> f64 {
    if time_map.len() < 2 {
        return 1.0;
    }
    let xs: Vec<f64> = time_map.iter().map(|p| p.tau_physical).collect();
    let ys: Vec<f64> = time_map.iter().map(|p| p.tau_uniform).collect();
    fit_affine(&xs, &ys).2
}

fn build_time_map(tick_indices: &[usize]) -> Vec<TimeMapPoint> {
    tick_indices
        .iter()
        .enumerate()
        .map(|(k_physical, &uniform_idx)| TimeMapPoint {
            tau_uniform: uniform_idx as f64,
            tau_physical: k_physical as f64,
        })
        .collect()
}

pub fn build_modular_multi_clock(
    hamiltonian: &crate::quantum::Hamiltonian,
    initial: QuantumState,
    reference: QuantumState,
    config: &ModularMultiClockConfig,
    kind: &str,
    label: &str,
) -> ModularMultiClockResult {
    let sites = config.n;
    let defect_site = config.defect_site.unwrap_or(sites / 2);
    let clock_sites = config.clock_sites.clone().unwrap_or_else(|| match kind {
        "grid" => {
            let cols = (sites as f64).sqrt().round() as usize;
            let rows = sites / cols.max(1);
            default_clock_sites_grid(rows, cols, defect_site)
        }
        "cube" => default_clock_sites_cube(sites, defect_site),
        _ => default_clock_sites_chain(sites),
    });

    let trajectory = evolve_trajectory(
        hamiltonian,
        &initial,
        Some(&reference),
        config.dt,
        config.steps,
        6,
    );

    let uniform_track = uniform_times(trajectory.len());
    let mut phys_tracks: Vec<Vec<f64>> = Vec::with_capacity(clock_sites.len());
    let mut clocks = Vec::with_capacity(clock_sites.len());

    for &site in &clock_sites {
        let region = vec![site];
        let modular_tau = cumulative_modular_time(&trajectory, &region);
        let tick_indices = event_ticks_from_cumulative(&modular_tau, config.modular_slices);
        let time_map = build_time_map(&tick_indices);
        let track: Vec<f64> = (0..trajectory.len())
            .map(|k| physical_time_at_uniform(&tick_indices, k))
            .collect();
        let sync = sync_r2_time_map(&time_map);
        let (slope, _, _) = fit_affine(
            &time_map.iter().map(|p| p.tau_physical).collect::<Vec<_>>(),
            &time_map.iter().map(|p| p.tau_uniform).collect::<Vec<_>>(),
        );
        clocks.push(ModularClockReading {
            site,
            label: if site == defect_site {
                format!("Site {site} (defect)")
            } else if site == 0 || site == sites.saturating_sub(1) {
                format!("Site {site} (edge)")
            } else {
                format!("Site {site}")
            },
            modular_tau,
            time_map,
            sync_r2_vs_uniform: sync,
            sync_slope_vs_uniform: slope,
        });
        phys_tracks.push(track);
    }

    let mut labels = vec!["Uniform (Δt)".to_string()];
    labels.extend(clocks.iter().map(|c| c.label.clone()));

    let n_clocks = 1 + clock_sites.len();
    let mut pairwise_r2 = vec![vec![1.0; n_clocks]; n_clocks];
    let mut all_tracks: Vec<&[f64]> = vec![&uniform_track];
    for t in &phys_tracks {
        all_tracks.push(t);
    }

    for i in 0..n_clocks {
        for j in (i + 1)..n_clocks {
            let r2 = if i == 0 {
                clocks[j - 1].sync_r2_vs_uniform
            } else if j == 0 {
                clocks[i - 1].sync_r2_vs_uniform
            } else {
                pairwise_sync_r2(all_tracks[i], all_tracks[j])
            };
            pairwise_r2[i][j] = r2;
            pairwise_r2[j][i] = r2;
        }
    }

    let mut min_pairwise_r2 = 1.0_f64;
    let mut inconsistent_pairs = 0usize;
    const THRESH: f64 = 0.95;
    for i in 0..n_clocks {
        for j in (i + 1)..n_clocks {
            let r2 = pairwise_r2[i][j];
            min_pairwise_r2 = min_pairwise_r2.min(r2);
            if r2 < THRESH {
                inconsistent_pairs += 1;
            }
        }
    }

    let defect_idx = clock_sites
        .iter()
        .position(|&s| s == defect_site)
        .map(|i| i + 1)
        .unwrap_or(1);
    let defect_uniform_r2 = pairwise_r2[0][defect_idx];

    let edge_a = config
        .edge_sites
        .map(|e| e[0])
        .or_else(|| clock_sites.first().copied())
        .and_then(|s| clock_sites.iter().position(|&c| c == s).map(|i| i + 1));
    let edge_b = config
        .edge_sites
        .map(|e| e[1])
        .or_else(|| clock_sites.last().copied())
        .and_then(|s| clock_sites.iter().position(|&c| c == s).map(|i| i + 1));
    let edge_edge_r2 = match (edge_a, edge_b) {
        (Some(a), Some(b)) if a != b => pairwise_r2[a][b],
        _ => 1.0,
    };

    ModularMultiClockResult {
        kind: kind.to_string(),
        label: label.to_string(),
        sites,
        defect_site,
        labels,
        clocks,
        pairwise_r2,
        min_pairwise_r2,
        defect_uniform_r2,
        edge_edge_r2,
        inconsistent_pairs,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{tfim_chain, tfim_grid};
    use crate::quantum::QuantumState;

    fn defect_reference(n: usize, center: usize) -> (QuantumState, QuantumState) {
        let mut initial = QuantumState::zero(n);
        initial.data[2 * (1 << center)] = 1.0;
        let mut reference = QuantumState::zero(n);
        reference.data[0] = 1.0;
        (initial, reference)
    }

    #[test]
    fn modular_multi_clock_chain_not_globally_consistent() {
        let n = 9;
        let model = tfim_chain(n, 1.0, 1.0);
        let (initial, reference) = defect_reference(n, n / 2);
        let result = build_modular_multi_clock(
            &model.hamiltonian,
            initial,
            reference,
            &ModularMultiClockConfig {
                n,
                dt: 0.2,
                steps: 40,
                clock_sites: Some(vec![0, n / 2, n - 1]),
                modular_slices: 12,
                defect_site: Some(n / 2),
                edge_sites: Some([0, n - 1]),
            },
            "chain",
            "TFIM chain",
        );
        assert!(
            result.defect_uniform_r2 > 0.85,
            "defectUniform={:.3}",
            result.defect_uniform_r2
        );
        assert!(
            result.min_pairwise_r2 < 0.95,
            "minPair={:.3}",
            result.min_pairwise_r2
        );
    }

    #[test]
    fn modular_multi_clock_grid_not_globally_consistent() {
        let rows = 3;
        let cols = 3;
        let sites = rows * cols;
        let model = tfim_grid(rows, cols, 1.0, 1.0);
        let defect_site = (rows / 2) * cols + cols / 2;
        let (initial, reference) = defect_reference(sites, defect_site);
        let result = build_modular_multi_clock(
            &model.hamiltonian,
            initial,
            reference,
            &ModularMultiClockConfig {
                n: sites,
                dt: 0.2,
                steps: 40,
                clock_sites: None,
                modular_slices: 12,
                defect_site: Some(defect_site),
                edge_sites: Some([0, sites - 1]),
            },
            "grid",
            "TFIM grid",
        );
        assert!(
            result.min_pairwise_r2 < 0.95,
            "minPair={:.3}",
            result.min_pairwise_r2
        );
        assert!(
            result.defect_uniform_r2 < 0.95,
            "modular grid clocks should desync from uniform Δt, defectUniform={:.3}",
            result.defect_uniform_r2
        );
    }
}
