//! Phase 6 matter/classicality runners.

use crate::branch_born::{run_branch_born_study, BranchBornResult};
use crate::decoherence::{chain_conditional_state, decoherence_final_state, DecoherenceQuenchConfig};
use crate::excitation_subspace::{
    run_excitation_subspace_probe, ExcitationSubspaceConfig, ExcitationSubspaceResult,
};
use crate::particle_stability::{run_particle_stability, ParticleStabilityResult};
use crate::stabilizer_search::{
    distance_scales_with_window, search_stabilizers, StabilizerSearchReport,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StabilizerSearchConfig {
    #[serde(flatten)]
    pub excitation: ExcitationSubspaceConfig,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StabilizerSearchResult {
    pub excitation: ExcitationSubspaceResult,
    pub stabilizer: StabilizerSearchReport,
    pub distance_scales: bool,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

pub fn run_stabilizer_search(config: &StabilizerSearchConfig) -> StabilizerSearchResult {
    let excitation = run_excitation_subspace_probe(&config.excitation);
    let deco_config = DecoherenceQuenchConfig {
        n: config.excitation.n,
        field: config.excitation.field,
        dt: config.excitation.dt,
        steps: config.excitation.steps,
        couple_step: config.excitation.couple_step,
        coupling: config.excitation.coupling,
        seed: config.excitation.seed,
    };
    let (psi, n_chain, env_q) = decoherence_final_state(&deco_config);
    let branch0 = chain_conditional_state(&psi, n_chain, env_q, 0);
    let branch1 = chain_conditional_state(&psi, n_chain, env_q, 1);
    let window = excitation.window_sites.clone();
    let stabilizer = search_stabilizers(
        &psi,
        n_chain,
        env_q,
        &branch0,
        &branch1,
        &window,
    );
    let distance_scales = distance_scales_with_window(
        &psi,
        n_chain,
        env_q,
        &branch0,
        &branch1,
        excitation.excitation_peak,
        config.excitation.window_radius,
        config.excitation.window_radius + 1,
        n_chain,
    );

    StabilizerSearchResult {
        excitation,
        stabilizer,
        distance_scales,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleStabilityConfig {
    pub n: usize,
    pub dt: f64,
    pub steps: usize,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticleStabilityRunResult {
    #[serde(flatten)]
    pub inner: ParticleStabilityResult,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

pub fn run_particle_stability_probe(config: &ParticleStabilityConfig) -> ParticleStabilityRunResult {
    ParticleStabilityRunResult {
        inner: run_particle_stability(config.n, config.dt, config.steps),
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchBornConfig {
    pub n: usize,
    pub field: f64,
    pub dt: f64,
    pub steps: usize,
    pub couple_step: usize,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchBornRunResult {
    #[serde(flatten)]
    pub inner: BranchBornResult,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

pub fn run_branch_born_probe(config: &BranchBornConfig) -> BranchBornRunResult {
    BranchBornRunResult {
        inner: run_branch_born_study(
            config.n,
            config.field,
            config.dt,
            config.steps,
            config.couple_step,
        ),
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EftDimensionConfig {
    #[serde(flatten)]
    pub excitation: ExcitationSubspaceConfig,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EftDimensionResult {
    pub window_sites: usize,
    pub measured_dof_per_site: f64,
    pub predicted_dof_per_site: f64,
    pub relative_error: f64,
    pub dof_agreement: bool,
    pub stabilizer: StabilizerSearchReport,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

pub fn run_eft_dimension_probe(config: &EftDimensionConfig) -> EftDimensionResult {
    let stab = run_stabilizer_search(&StabilizerSearchConfig {
        excitation: config.excitation.clone(),
    });
    let window_len = stab.stabilizer.window_sites.len().max(1);
    let branch_rank_avg = (stab.excitation.branch0_effective_rank
        + stab.excitation.branch1_effective_rank)
        / 2.0;
    let measured_dof_per_site = branch_rank_avg / window_len as f64;
    let predicted_dof_per_site = stab.stabilizer.code_rate;
    let relative_error = if predicted_dof_per_site > 1e-6 {
        ((measured_dof_per_site - predicted_dof_per_site) / predicted_dof_per_site).abs()
    } else {
        measured_dof_per_site
    };
    let dof_agreement = relative_error < 0.75 || (measured_dof_per_site - predicted_dof_per_site).abs() < 0.5;

    EftDimensionResult {
        window_sites: window_len,
        measured_dof_per_site,
        predicted_dof_per_site,
        relative_error,
        dof_agreement,
        stabilizer: stab.stabilizer,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}
