//! Holography and RT mass tests — port of `src/sim/holography.ts`.

use crate::geometry::{entropy_of_region, mutual_information_matrix};
use crate::quantum::{evolve_interval, kick_x, Hamiltonian, QuantumState};
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AreaLawPoint {
    pub size: usize,
    pub s_ground: f64,
    pub s_random: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RtPoint {
    pub entropy: f64,
    pub cut: f64,
    pub size: usize,
    pub start: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dist_from_mass: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RtFit {
    pub rt_points: Vec<RtPoint>,
    pub rt_slope: f64,
    pub rt_r2: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HolographyReport {
    pub n: usize,
    pub area_law: Vec<AreaLawPoint>,
    pub rt_points: Vec<RtPoint>,
    pub rt_slope: f64,
    pub rt_r2: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DensitySweepPoint {
    pub count: usize,
    pub density: f64,
    pub slope: f64,
    pub r2: f64,
    pub delta_slope: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RtMassReport {
    pub n: usize,
    pub mass_site: usize,
    pub strength: f64,
    pub vacuum: RtFit,
    pub mass: RtFit,
    pub sweep: Vec<SweepPoint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub density_sweep: Option<Vec<DensitySweepPoint>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SweepPoint {
    pub strength: f64,
    pub slope: f64,
    pub r2: f64,
}

fn fit_rt_through_origin(points: &[RtPoint]) -> (f64, f64) {
    let mut sxy = 0.0;
    let mut sxx = 0.0;
    let mut sy = 0.0;
    for p in points {
        sxy += p.cut * p.entropy;
        sxx += p.cut * p.cut;
        sy += p.entropy;
    }
    let rt_slope = if sxx > 0.0 { sxy / sxx } else { 0.0 };
    let y_bar = if !points.is_empty() {
        sy / points.len() as f64
    } else {
        0.0
    };
    let mut ss_res = 0.0;
    let mut ss_tot = 0.0;
    for p in points {
        let pred = rt_slope * p.cut;
        ss_res += (p.entropy - pred).powi(2);
        ss_tot += (p.entropy - y_bar).powi(2);
    }
    let rt_r2 = if ss_tot > 0.0 { 1.0 - ss_res / ss_tot } else { 0.0 };
    (rt_slope, rt_r2)
}

fn region_dist_from_mass(region: &[usize], mass_site: usize) -> f64 {
    region
        .iter()
        .map(|&q| ((q as i32) - (mass_site as i32)).abs() as f64)
        .fold(f64::INFINITY, f64::min)
}

pub fn analyze_rt_relation(state: &QuantumState, mass_site: Option<usize>) -> RtFit {
    let n = state.n;
    let mi = mutual_information_matrix(state);
    let mut rt_points = Vec::new();

    for start in 0..n {
        for l in 1..(n - start) {
            let region: Vec<usize> = (start..(start + l)).collect();
            let mut in_region = vec![false; n];
            for &q in &region {
                in_region[q] = true;
            }
            let mut cut = 0.0;
            for &a in &region {
                for b in 0..n {
                    if !in_region[b] {
                        cut += mi[a][b];
                    }
                }
            }
            cut *= 0.5;
            rt_points.push(RtPoint {
                entropy: entropy_of_region(state, &region),
                cut,
                size: l,
                start,
                dist_from_mass: mass_site.map(|m| region_dist_from_mass(&region, m)),
            });
        }
    }

    let (rt_slope, rt_r2) = fit_rt_through_origin(&rt_points);
    RtFit {
        rt_points,
        rt_slope,
        rt_r2,
    }
}

pub fn analyze_holography(ground: &QuantumState, random: &QuantumState) -> HolographyReport {
    let n = ground.n;
    let RtFit {
        rt_points,
        rt_slope,
        rt_r2,
    } = analyze_rt_relation(ground, None);

    let mut area_law = Vec::new();
    for l in 1..n {
        let region: Vec<usize> = (0..l).collect();
        area_law.push(AreaLawPoint {
            size: l,
            s_ground: entropy_of_region(ground, &region),
            s_random: entropy_of_region(random, &region),
        });
    }

    HolographyReport {
        n,
        area_law,
        rt_points,
        rt_slope,
        rt_r2,
    }
}

pub fn inject_mass(
    h: &Hamiltonian,
    ground: &QuantumState,
    mass_site: usize,
    strength: f64,
) -> QuantumState {
    inject_mass_multi(h, ground, &[mass_site], strength)
}

/// Evenly spaced excitation sites in the inner half of the chain.
pub fn mass_sites_evenly(n: usize, count: usize) -> Vec<usize> {
    let count = count.clamp(1, n);
    if count == 1 {
        return vec![n / 2];
    }
    let lo = n / 4;
    let hi = n.saturating_sub(1).saturating_sub(n / 4);
    if hi <= lo {
        return (0..count).map(|i| (i * n / count).min(n - 1)).collect();
    }
    (0..count)
        .map(|i| {
            lo + ((hi - lo) as f64 * i as f64 / (count - 1) as f64).round() as usize
        })
        .collect()
}

pub fn inject_mass_multi(
    h: &Hamiltonian,
    ground: &QuantumState,
    sites: &[usize],
    strength: f64,
) -> QuantumState {
    if strength <= 0.0 || sites.is_empty() {
        return ground.clone_state();
    }
    let mut rng = Rng::new(42);
    let radius = h.estimate_spectral_radius(&mut rng, 30).max(1e-6);
    let mut psi = ground.clone_state();
    for &site in sites {
        psi = kick_x(&psi, site.min(ground.n.saturating_sub(1)));
    }
    psi.normalize();
    psi = evolve_interval(h, &psi, strength, radius, 6);
    psi.normalize();
    psi
}

pub fn analyze_rt_mass_density_sweep(
    h: &Hamiltonian,
    ground: &QuantumState,
    strength: f64,
    max_count: usize,
) -> Vec<DensitySweepPoint> {
    let n = ground.n;
    let vacuum_slope = analyze_rt_relation(ground, Some(n / 2)).rt_slope;
    let max_count = max_count.clamp(1, n.min(8));
    let mut points = Vec::new();
    for count in 1..=max_count {
        let sites = mass_sites_evenly(n, count);
        let state = inject_mass_multi(h, ground, &sites, strength);
        let fit = analyze_rt_relation(&state, Some(n / 2));
        points.push(DensitySweepPoint {
            count,
            density: count as f64 / n as f64,
            slope: fit.rt_slope,
            r2: fit.rt_r2,
            delta_slope: fit.rt_slope - vacuum_slope,
        });
    }
    points
}

pub fn analyze_rt_mass_deformation(
    h: &Hamiltonian,
    ground: &QuantumState,
    mass_site: usize,
    strength: f64,
    sweep_steps: usize,
    max_strength: f64,
    density_sweep: bool,
    max_mass_count: usize,
) -> RtMassReport {
    let n = ground.n;
    let vacuum = analyze_rt_relation(ground, Some(mass_site));
    let mass_state = inject_mass(h, ground, mass_site, strength);
    let mass = analyze_rt_relation(&mass_state, Some(mass_site));

    let mut sweep = vec![SweepPoint {
        strength: 0.0,
        slope: vacuum.rt_slope,
        r2: vacuum.rt_r2,
    }];
    for i in 1..sweep_steps {
        let s = max_strength * i as f64 / (sweep_steps - 1) as f64;
        let st = inject_mass(h, ground, mass_site, s);
        let fit = analyze_rt_relation(&st, Some(mass_site));
        sweep.push(SweepPoint {
            strength: s,
            slope: fit.rt_slope,
            r2: fit.rt_r2,
        });
    }

    RtMassReport {
        n,
        mass_site,
        strength,
        vacuum,
        mass,
        sweep,
        density_sweep: if density_sweep {
            Some(analyze_rt_mass_density_sweep(
                h,
                ground,
                strength,
                max_mass_count,
            ))
        } else {
            None
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::tfim_chain;
    use crate::quantum::ground_state;
    use crate::rng::Rng;

    #[test]
    fn rt_slope_rises_with_excitation_density() {
        let model = tfim_chain(10, 1.0, 1.5);
        let mut rng = Rng::new(7);
        let (ground, _, _) = ground_state(&model.hamiltonian, &mut rng, 4000, 1e-9);
        let sweep = analyze_rt_mass_density_sweep(&model.hamiltonian, &ground, 1.0, 5);
        assert!(sweep.len() >= 3);
        let last = sweep.last().unwrap().delta_slope;
        let first = sweep.first().unwrap().delta_slope;
        assert!(
            last > first + 0.05,
            "Δslope should grow with density: first={first} last={last}"
        );
    }
}
