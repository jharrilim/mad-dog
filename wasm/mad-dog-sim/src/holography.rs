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
pub struct RtMassReport {
    pub n: usize,
    pub mass_site: usize,
    pub strength: f64,
    pub vacuum: RtFit,
    pub mass: RtFit,
    pub sweep: Vec<SweepPoint>,
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
    if strength <= 0.0 {
        return ground.clone_state();
    }
    let mut rng = Rng::new(42);
    let radius = h.estimate_spectral_radius(&mut rng, 30).max(1e-6);
    let mut psi = kick_x(ground, mass_site);
    psi.normalize();
    psi = evolve_interval(h, &psi, strength, radius, 6);
    psi.normalize();
    psi
}

pub fn analyze_rt_mass_deformation(
    h: &Hamiltonian,
    ground: &QuantumState,
    mass_site: usize,
    strength: f64,
    sweep_steps: usize,
    max_strength: f64,
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
    }
}
