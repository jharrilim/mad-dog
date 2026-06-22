//! 3+1 universe lab runner for WASM.

use crate::geometry::{analyze_emergent_geometry, EmergenceReport};
use crate::models::{cube_edges, tfim_cube, BuiltModel, TruePosition};
use crate::quantum::{evolve_interval, QuantumState};
use crate::rng::Rng;
use crate::spacetime::{build_spacetime, measure_light_cone, LightCone, SpacetimeConfig, SpacetimeResult};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Universe3DConfig {
    pub lx: usize,
    pub ly: usize,
    pub lz: usize,
    pub field: f64,
    pub dt: f64,
    pub steps: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniverseModelSummary {
    pub label: String,
    pub layout: UniverseLayoutSummary,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniverseLayoutSummary {
    pub true_positions: Vec<TruePosition>,
    pub expected_dim: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Universe3DResult {
    pub model: UniverseModelSummary,
    pub spacetime: SpacetimeResult,
    pub light_cone: LightCone,
    pub defect_site: usize,
    pub edges: Vec<[usize; 2]>,
    pub site_distances: Vec<f64>,
    pub elapsed_ms: f64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniverseSliceConfig {
    pub lx: usize,
    pub ly: usize,
    pub lz: usize,
    pub field: f64,
    pub dt: f64,
    pub k: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniverseSliceResult {
    pub report: EmergenceReport,
    pub defect_site: usize,
    pub elapsed_ms: f64,
}

fn cube_defect_site(lx: usize, ly: usize, lz: usize) -> usize {
    let cx = lx / 2;
    let cy = ly / 2;
    let cz = lz / 2;
    cz * (lx * ly) + cy * lx + cx
}

fn lattice_distances(positions: &[TruePosition], center: usize) -> Vec<f64> {
    let c = &positions[center];
    let cz = c.z.unwrap_or(0.0);
    positions
        .iter()
        .map(|p| {
            (p.x - c.x).abs() + (p.y - c.y).abs() + (p.z.unwrap_or(0.0) - cz).abs()
        })
        .collect()
}

fn defect_initial(n: usize, defect_site: usize) -> QuantumState {
    let mut initial = QuantumState::zero(n);
    initial.data[2 * (1 << defect_site)] = 1.0;
    initial
}

fn reference_initial(n: usize) -> QuantumState {
    let mut reference = QuantumState::zero(n);
    reference.data[0] = 1.0;
    reference
}

fn model_summary(model: &BuiltModel) -> UniverseModelSummary {
    UniverseModelSummary {
        label: model.label.clone(),
        layout: UniverseLayoutSummary {
            true_positions: model.layout.true_positions.clone(),
            expected_dim: model.layout.expected_dim,
        },
    }
}

pub fn run_universe_3d(config: &Universe3DConfig) -> Universe3DResult {
    let model = tfim_cube(config.lx, config.ly, config.lz, 1.0, config.field);
    let n = model.hamiltonian.n;
    let defect_site = cube_defect_site(config.lx, config.ly, config.lz);
    let initial = defect_initial(n, defect_site);
    let reference = reference_initial(n);
    let site_distances = lattice_distances(&model.layout.true_positions, defect_site);
    let spacetime = build_spacetime(SpacetimeConfig {
        hamiltonian: &model.hamiltonian,
        initial,
        reference: Some(reference),
        dt: config.dt,
        steps: config.steps,
        embed_dim: 3,
        align_to: Some(&model.layout.true_positions),
        order: 6,
        include_geometry: true,
        track_energy: true,
        retain_slices: true,
        worldline_defects: None,
        track_worldline: true,
        defect_site: Some(defect_site),
        seed: 42,
    });
    let light_cone = measure_light_cone(&spacetime, 0.12, Some(defect_site), Some(&site_distances));
    Universe3DResult {
        model: model_summary(&model),
        spacetime,
        light_cone,
        defect_site,
        edges: cube_edges(config.lx, config.ly, config.lz),
        site_distances,
        elapsed_ms: 0.0,
    }
}

pub fn run_universe_slice(config: &UniverseSliceConfig) -> UniverseSliceResult {
    let model = tfim_cube(config.lx, config.ly, config.lz, 1.0, config.field);
    let defect_site = cube_defect_site(config.lx, config.ly, config.lz);
    let n = model.hamiltonian.n;
    let initial = defect_initial(n, defect_site);
    let mut rng = Rng::new(42);
    let radius = model
        .hamiltonian
        .estimate_spectral_radius(&mut rng, 30)
        .max(1e-6);
    let mut psi = initial;
    for _ in 0..config.k {
        psi = evolve_interval(&model.hamiltonian, &psi, config.dt, radius, 6);
        psi.normalize();
    }
    let report = analyze_emergent_geometry(&psi, 1.0, 3);
    UniverseSliceResult {
        report,
        defect_site,
        elapsed_ms: 0.0,
    }
}
