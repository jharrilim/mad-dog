//! Emergent spacetime — port of `src/sim/spacetime.ts`.

use crate::geometry::analyze_emergent_geometry;
use crate::linalg::{procrustes_2d, procrustes_3d};
use crate::models::TruePosition;
use crate::quantum::{
    evolve_interval_inplace, expectation_z_all, signal_from_z, EvolveScratch, Hamiltonian,
    QuantumState,
};
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpacetimeSlice {
    pub k: usize,
    pub t: f64,
    pub coords: Vec<Vec<f64>>,
    pub z_expectation: Vec<f64>,
    pub signal: Vec<f64>,
    pub energy: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpacetimeResult {
    pub sites: usize,
    pub slices: Vec<SpacetimeSlice>,
    pub energy_drift: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub light_cone: Option<LightCone>,
    /// Signal-weighted centroid track for a single defect quench.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worldline: Option<Vec<WorldlinePoint>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worldlines: Option<[Vec<WorldlinePoint>; 2]>,
    pub elapsed_ms: f64,
}

#[derive(Clone, Debug)]
pub struct SpacetimeConfig<'a> {
    pub hamiltonian: &'a Hamiltonian,
    pub initial: QuantumState,
    pub reference: Option<QuantumState>,
    pub dt: f64,
    pub steps: usize,
    pub embed_dim: usize,
    pub align_to: Option<&'a [TruePosition]>,
    pub order: usize,
    /// When false, skip MI/MDS per slice (light-cone signal only).
    pub include_geometry: bool,
    /// When false, skip ⟨H⟩ each slice (scattering sweeps).
    pub track_energy: bool,
    /// When false, omit slice history (lite scattering).
    pub retain_slices: bool,
    /// Inline worldline tracking for lite scattering (defect sites on left/right).
    pub worldline_defects: Option<[usize; 2]>,
    /// Track a single excitation (centroid/peak); requires defect site for half-chain split at center.
    pub track_worldline: bool,
    pub defect_site: Option<usize>,
    /// When set, subtract ⟨Z_ref⟩ at this site from every site's signal (clock-subset observer).
    pub signal_reference_site: Option<usize>,
    /// RNG seed for spectral-radius estimate.
    pub seed: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldlinePoint {
    pub t: f64,
    pub site: usize,
    pub amplitude: f64,
}

/// Peak-signal track for a single defect. On 1D chains uses a half-window so
/// symmetric center quenches follow one wavefront; on higher-D embeddings uses
/// global peak (site index is not a spatial axis).
pub fn track_single_worldline(
    signal: &[f64],
    defect_site: usize,
    n: usize,
    embed_dim: usize,
    t: f64,
) -> WorldlinePoint {
    let (lo, hi) = if embed_dim >= 2 {
        (0usize, n - 1)
    } else {
        let mid = n / 2;
        if defect_site < mid {
            (0, mid)
        } else if defect_site > mid {
            (mid, n - 1)
        } else {
            ((mid + 1).min(n - 1), n - 1)
        }
    };
    let mut best_site = defect_site.min(n - 1);
    let mut best_amp = -1.0_f64;
    for i in lo..=hi {
        let amp = signal[i];
        if amp > best_amp {
            best_amp = amp;
            best_site = i;
        }
    }
    WorldlinePoint {
        t,
        site: best_site,
        amplitude: best_amp.max(0.0),
    }
}

fn track_worldline_point(
    signal: &[f64],
    defect_site: usize,
    n: usize,
    side: &str,
    t: f64,
) -> WorldlinePoint {
    let mid = n / 2;
    let (lo, hi) = if side == "left" {
        (0usize, mid.saturating_sub(1))
    } else {
        (mid, n - 1)
    };
    let mut best_site = defect_site;
    let mut best_amp = -1.0;
    for i in lo..=hi {
        let amp = signal[i];
        if amp > best_amp {
            best_amp = amp;
            best_site = i;
        }
    }
    WorldlinePoint {
        t,
        site: best_site,
        amplitude: best_amp,
    }
}

fn orient_coords(coords: &[f64]) -> Vec<f64> {
    let n = coords.len();
    if n < 2 {
        return coords.to_vec();
    }
    let mean_i = (n - 1) as f64 / 2.0;
    let mean_c = coords.iter().sum::<f64>() / n as f64;
    let mut cov = 0.0;
    for (i, &c) in coords.iter().enumerate() {
        cov += (i as f64 - mean_i) * (c - mean_c);
    }
    if cov < 0.0 {
        coords.iter().map(|c| -c).collect()
    } else {
        coords.to_vec()
    }
}

pub fn build_spacetime(config: SpacetimeConfig<'_>) -> SpacetimeResult {
    let hamiltonian = config.hamiltonian;
    let sites = hamiltonian.n;
    let embed_dim = config.embed_dim.max(1);
    let use_3d = embed_dim >= 3;
    let use_2d = embed_dim >= 2 && !use_3d;

    let align_target_3: Option<Vec<Vec<f64>>> = config.align_to.map(|pts| {
        pts.iter()
            .map(|p| vec![p.x, p.y, p.z.unwrap_or(0.0)])
            .collect()
    });
    let align_target_2: Option<Vec<Vec<f64>>> = config.align_to.map(|pts| {
        pts.iter().map(|p| vec![p.x, p.y]).collect()
    });

    let energy0 = if config.track_energy {
        hamiltonian.expectation(&config.initial)
    } else {
        0.0
    };
    let mut energy_drift = 0.0_f64;
    let mut rng = Rng::new(config.seed);
    let radius = hamiltonian.spectral_radius(&mut rng);

    let mut slices = Vec::new();
    let mut psi = config.initial.clone_state();
    let mut ref_state = config.reference.clone();
    let base_z = expectation_z_all(&config.initial);
    let mut scratch = EvolveScratch::new(psi.dim);
    let mut ref_scratch = if ref_state.is_some() {
        Some(EvolveScratch::new(psi.dim))
    } else {
        None
    };
    let mut worldlines = config.worldline_defects.map(|d| {
        (
            Vec::with_capacity(config.steps),
            Vec::with_capacity(config.steps),
            d,
        )
    });
    let mut worldline = if config.track_worldline {
        Some(Vec::with_capacity(config.steps))
    } else {
        None
    };

    for k in 0..config.steps {
        let t = k as f64 * config.dt;
        let energy = if config.track_energy {
            hamiltonian.expectation_with_scratch(&psi, &mut scratch.h_psi)
        } else {
            0.0
        };
        if config.track_energy {
            energy_drift = energy_drift.max((energy - energy0).abs());
        }

        let z_expectation = expectation_z_all(&psi);
        let ref_z: Vec<f64> = if let Some(site) = config.signal_reference_site {
            if let Some(ref r) = ref_state {
                let rz = crate::quantum::expectation_z(r, site);
                vec![rz; sites]
            } else {
                let rz = base_z[site.min(sites.saturating_sub(1))];
                vec![rz; sites]
            }
        } else if let Some(ref r) = ref_state {
            expectation_z_all(r)
        } else {
            base_z.clone()
        };
        let signal = signal_from_z(&z_expectation, &ref_z);

        if let Some((ref mut wl1, ref mut wl2, defects)) = worldlines.as_mut() {
            if embed_dim >= 2 {
                wl1.push(track_single_worldline(
                    &signal,
                    defects[0],
                    sites,
                    embed_dim,
                    t,
                ));
                wl2.push(track_single_worldline(
                    &signal,
                    defects[1],
                    sites,
                    embed_dim,
                    t,
                ));
            } else {
                wl1.push(track_worldline_point(&signal, defects[0], sites, "left", t));
                wl2.push(track_worldline_point(&signal, defects[1], sites, "right", t));
            }
        }
        if let Some(ref mut wl) = worldline {
            if let Some(defect) = config.defect_site {
                wl.push(track_single_worldline(
                    &signal,
                    defect,
                    sites,
                    config.embed_dim,
                    t,
                ));
            }
        }

        if config.retain_slices {
            let coords = if config.include_geometry {
            let report = analyze_emergent_geometry(&psi, 1.0, embed_dim.max(1));
            if use_3d {
                let raw: Vec<Vec<f64>> = report
                    .mds
                    .coords
                    .iter()
                    .map(|c| {
                        vec![
                            c.first().copied().unwrap_or(0.0),
                            c.get(1).copied().unwrap_or(0.0),
                            c.get(2).copied().unwrap_or(0.0),
                        ]
                    })
                    .collect();
                if let Some(ref target) = align_target_3 {
                    procrustes_3d(&raw, target)
                } else {
                    raw
                }
            } else if use_2d {
                let raw: Vec<Vec<f64>> = report
                    .mds
                    .coords
                    .iter()
                    .map(|c| vec![c.first().copied().unwrap_or(0.0), c.get(1).copied().unwrap_or(0.0)])
                    .collect();
                if let Some(ref target) = align_target_2 {
                    procrustes_2d(&raw, target)
                } else {
                    raw
                }
            } else {
                let oriented = orient_coords(
                    &report
                        .mds
                        .coords
                        .iter()
                        .map(|c| c.first().copied().unwrap_or(0.0))
                        .collect::<Vec<_>>(),
                );
                oriented.into_iter().map(|x| vec![x]).collect()
            }
            } else {
                vec![Vec::new(); sites]
            };

            slices.push(SpacetimeSlice {
                k,
                t,
                coords,
                z_expectation,
                signal,
                energy,
            });
        }

        evolve_interval_inplace(hamiltonian, &mut psi, config.dt, radius, config.order, &mut scratch);
        psi.normalize();
        if let Some(ref mut r) = ref_state {
            evolve_interval_inplace(
                hamiltonian,
                r,
                config.dt,
                radius,
                config.order,
                ref_scratch.as_mut().unwrap(),
            );
            r.normalize();
        }
    }

    SpacetimeResult {
        sites,
        slices,
        energy_drift,
        light_cone: None,
        worldline,
        worldlines: worldlines.map(|(a, b, _)| [a, b]),
        elapsed_ms: 0.0,
    }
}

/// Light-cone trajectory without per-slice MI/MDS (scattering, sweeps).
pub fn build_light_cone_trajectory(mut config: SpacetimeConfig<'_>) -> SpacetimeResult {
    config.include_geometry = false;
    build_spacetime(config)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LightCone {
    pub velocity: f64,
    pub arrivals: Vec<f64>,
    pub center: usize,
    pub dt: f64,
}

pub fn measure_light_cone(
    result: &SpacetimeResult,
    threshold_fraction: f64,
    center_site: Option<usize>,
    site_distances: Option<&[f64]>,
) -> LightCone {
    let n = result.sites;
    let center = center_site.unwrap_or(n / 2);
    let mut max = 0.0_f64;
    for s in &result.slices {
        for &v in &s.signal {
            max = max.max(v);
        }
    }
    let thr = (max * threshold_fraction).max(1e-6);

    let mut arrivals = vec![f64::INFINITY; n];
    for slice in &result.slices {
        for (i, &d) in slice.signal.iter().enumerate() {
            if d > thr && slice.t < arrivals[i] {
                arrivals[i] = slice.t;
            }
        }
    }

    let mut num = 0.0;
    let mut den = 0.0;
    for i in 0..n {
        if i == center {
            continue;
        }
        let t = arrivals[i];
        if !t.is_finite() || t <= 0.0 {
            continue;
        }
        let d = site_distances
            .map(|sd| sd[i])
            .unwrap_or(((i as i32) - (center as i32)).abs() as f64);
        num += d * t;
        den += t * t;
    }
    let velocity = if den > 0.0 { num / den } else { 0.0 };
    let dt = if result.slices.len() > 1 {
        result.slices[1].t - result.slices[0].t
    } else {
        1.0
    };
    LightCone {
        velocity,
        arrivals,
        center,
        dt,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LightConeComparison {
    pub lattice: LightCone,
    pub mi: LightCone,
    pub mi_time_avg: LightCone,
    pub velocity_ratio: f64,
}

fn chain_distances(n: usize, center: usize) -> Vec<f64> {
    (0..n).map(|i| ((i as i32) - (center as i32)).abs() as f64).collect()
}

pub fn average_mi_distances_from_trajectory(
    hamiltonian: &crate::quantum::Hamiltonian,
    initial: &crate::quantum::QuantumState,
    center: usize,
    dt: f64,
    steps: usize,
    order: usize,
) -> Vec<f64> {
    use crate::geometry::mi_distances_from_state;
    use crate::quantum::evolve_interval_inplace;
    use crate::rng::Rng;

    let n = hamiltonian.n;
    let mut sums = vec![0.0; n];
    let mut psi = initial.clone();
    let mut rng = Rng::new(42);
    let radius = hamiltonian.spectral_radius(&mut rng);
    let mut scratch = crate::quantum::EvolveScratch::new(psi.dim);
    let count = steps.max(1) as f64;
    for k in 0..steps {
        let d = mi_distances_from_state(&psi, center, 1.0);
        for i in 0..n {
            sums[i] += d[i];
        }
        if k + 1 < steps {
            evolve_interval_inplace(hamiltonian, &mut psi, dt, radius, order, &mut scratch);
            psi.normalize();
        }
    }
    sums.iter().map(|s| s / count).collect()
}

pub fn compare_light_cone_velocities(
    result: &SpacetimeResult,
    threshold_fraction: f64,
    center_site: Option<usize>,
    mi_distances: Option<&[f64]>,
    mi_distances_time_avg: Option<&[f64]>,
) -> LightConeComparison {
    let center = center_site.unwrap_or(result.sites / 2);
    let lattice_d = chain_distances(result.sites, center);
    let lattice = measure_light_cone(
        result,
        threshold_fraction,
        Some(center),
        Some(&lattice_d),
    );
    let mi_d = mi_distances.unwrap_or(&lattice_d);
    let mi_time_d = mi_distances_time_avg.unwrap_or(mi_d);
    let mi = measure_light_cone(result, threshold_fraction, Some(center), Some(mi_d));
    let mi_time_avg =
        measure_light_cone(result, threshold_fraction, Some(center), Some(mi_time_d));
    let velocity_ratio = if lattice.velocity > 1e-9 {
        mi.velocity / lattice.velocity
    } else {
        0.0
    };
    LightConeComparison {
        lattice,
        mi,
        mi_time_avg,
        velocity_ratio,
    }
}
