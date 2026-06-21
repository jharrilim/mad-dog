//! Holography runners for WASM.

use crate::holography::{
    analyze_holography, analyze_rt_mass_deformation, HolographyReport, RtMassReport,
};
use crate::models::tfim_chain;
use crate::quantum::{ground_state, make_random_state};
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HolographyRunConfig {
    pub n: usize,
    pub field: f64,
    pub seed: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HolographyRunResult {
    pub label: String,
    pub energy: f64,
    pub report: HolographyReport,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RtMassRunConfig {
    pub n: usize,
    pub field: f64,
    pub seed: u32,
    pub mass_site: Option<usize>,
    pub strength: f64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RtMassRunResult {
    pub label: String,
    pub report: RtMassReport,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

pub fn run_holography(config: &HolographyRunConfig) -> HolographyRunResult {
    let model = tfim_chain(config.n, 1.0, config.field);
    let mut rng = Rng::new(config.seed);
    let (state, energy, _) = ground_state(&model.hamiltonian, &mut rng, 4000, 1e-9);
    let random = make_random_state(config.n, &mut Rng::new(config.seed.wrapping_add(4242)));
    let report = analyze_holography(&state, &random);
    HolographyRunResult {
        label: model.label,
        energy,
        report,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}

pub fn run_rt_mass(config: &RtMassRunConfig) -> RtMassRunResult {
    let model = tfim_chain(config.n, 1.0, config.field);
    let mut rng = Rng::new(config.seed);
    let (state, _, _) = ground_state(&model.hamiltonian, &mut rng, 4000, 1e-9);
    let mass_site = config.mass_site.unwrap_or(config.n / 2);
    let report = analyze_rt_mass_deformation(
        &model.hamiltonian,
        &state,
        mass_site,
        config.strength,
        9,
        2.0,
    );
    RtMassRunResult {
        label: model.label,
        report,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}
