//! N-clock consistency networks — extend relational time to multiple physical clocks.

use crate::quantum::{expectation_z, QuantumState};
use crate::relational_time::{
    evolve_trajectory, fit_affine, physical_clock_indices, physical_clock_uniform_r2,
    physical_time_at_uniform, TimeMapPoint,
};
use serde::{Deserialize, Serialize};

fn default_physical_slices() -> usize {
    15
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiClockConfig {
    pub n: usize,
    pub dt: f64,
    pub steps: usize,
    #[serde(default)]
    pub clock_sites: Option<Vec<usize>>,
    #[serde(default = "default_physical_slices")]
    pub physical_slices: usize,
    #[serde(default)]
    pub defect_site: Option<usize>,
    #[serde(default)]
    pub edge_sites: Option<[usize; 2]>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalClockReading {
    pub site: usize,
    pub label: String,
    pub time_map: Vec<TimeMapPoint>,
    pub sync_r2_vs_uniform: f64,
    pub sync_slope_vs_uniform: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiClockResult {
    pub kind: String,
    pub label: String,
    pub sites: usize,
    pub defect_site: usize,
    pub labels: Vec<String>,
    pub clocks: Vec<PhysicalClockReading>,
    pub pairwise_r2: Vec<Vec<f64>>,
    pub min_pairwise_r2: f64,
    pub defect_uniform_r2: f64,
    pub edge_edge_r2: f64,
    pub inconsistent_pairs: usize,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

fn default_clock_sites_chain(n: usize) -> Vec<usize> {
    let center = n / 2;
    vec![0, center, n.saturating_sub(1)]
}

fn default_clock_sites_grid(rows: usize, cols: usize, defect_site: usize) -> Vec<usize> {
    let corners = [0, cols - 1, (rows - 1) * cols, rows * cols - 1];
    if corners.contains(&defect_site) {
        vec![corners[0], corners[2], corners[3]]
    } else {
        vec![corners[0], defect_site, corners[3]]
    }
}

fn default_clock_sites_cube(sites: usize, defect_site: usize) -> Vec<usize> {
    vec![0, defect_site, sites - 1]
}

/// Interpolated physical tick reading at uniform slice `k`.
fn physical_time_at_uniform_local(phys_indices: &[usize], k: usize) -> f64 {
    physical_time_at_uniform(phys_indices, k)
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

fn build_time_map(phys_indices: &[usize]) -> Vec<TimeMapPoint> {
    phys_indices
        .iter()
        .enumerate()
        .map(|(k_physical, &uniform_idx)| TimeMapPoint {
            tau_uniform: uniform_idx as f64,
            tau_physical: k_physical as f64,
        })
        .collect()
}

pub fn build_multi_clock(
    hamiltonian: &crate::quantum::Hamiltonian,
    initial: QuantumState,
    reference: QuantumState,
    config: &MultiClockConfig,
    kind: &str,
    label: &str,
) -> MultiClockResult {
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

    let base_z: Vec<f64> = (0..sites)
        .map(|q| expectation_z(&initial, q))
        .collect();

    let uniform_track = uniform_times(trajectory.len());
    let mut phys_tracks: Vec<Vec<f64>> = Vec::with_capacity(clock_sites.len());
    let mut clocks = Vec::with_capacity(clock_sites.len());

    for &site in &clock_sites {
        let (indices, _) =
            physical_clock_indices(&trajectory, site, &base_z, config.physical_slices);
        let time_map = build_time_map(&indices);
        let track: Vec<f64> = (0..trajectory.len())
            .map(|k| physical_time_at_uniform_local(&indices, k))
            .collect();
        let sync = sync_r2_time_map(&time_map);
        let (slope, _, _) = fit_affine(
            &time_map.iter().map(|p| p.tau_physical).collect::<Vec<_>>(),
            &time_map.iter().map(|p| p.tau_uniform).collect::<Vec<_>>(),
        );
        clocks.push(PhysicalClockReading {
            site,
            label: if site == defect_site {
                format!("Site {site} (defect)")
            } else if site == 0 || site == sites.saturating_sub(1) {
                format!("Site {site} (edge)")
            } else {
                format!("Site {site}")
            },
            time_map,
            sync_r2_vs_uniform: sync,
            sync_slope_vs_uniform: slope,
        });
        phys_tracks.push(track);
    }

    let mut labels = vec!["Uniform (Δt)".to_string()];
    labels.extend(clocks.iter().map(|c| c.label.clone()));

    // Index 0 = uniform; 1.. = physical clocks.
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

    MultiClockResult {
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
    use crate::models::tfim_chain;
    use crate::quantum::QuantumState;

    fn defect_reference(n: usize, center: usize) -> (QuantumState, QuantumState) {
        let mut initial = QuantumState::zero(n);
        initial.data[2 * (1 << center)] = 1.0;
        let mut reference = QuantumState::zero(n);
        reference.data[0] = 1.0;
        (initial, reference)
    }

    #[test]
    fn multi_clock_network_not_globally_consistent() {
        let n = 9;
        let model = tfim_chain(n, 1.0, 1.0);
        let (initial, reference) = defect_reference(n, n / 2);
        let result = build_multi_clock(
            &model.hamiltonian,
            initial,
            reference,
            &MultiClockConfig {
                n,
                dt: 0.2,
                steps: 40,
                clock_sites: Some(vec![0, n / 2, n - 1]),
                physical_slices: 15,
                defect_site: Some(n / 2),
                edge_sites: Some([0, n - 1]),
            },
            "chain",
            "TFIM chain",
        );
        assert!(result.defect_uniform_r2 > 0.9);
        assert!(result.min_pairwise_r2 < 0.95);
    }
}
