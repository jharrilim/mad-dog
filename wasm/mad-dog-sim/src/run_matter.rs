//! Phase 6 matter/classicality runners.

use crate::branch_born::{run_branch_born_study, BranchBornResult};
use crate::decoherence::{chain_conditional_state, decoherence_final_state};
use crate::eft_dof::{independent_dof_per_site, two_of_three_dof_agreement};
use crate::excitation_subspace::{
    deco_config_from_excitation, run_excitation_subspace_probe, ExcitationSubspaceConfig,
    ExcitationSubspaceResult,
};
use crate::particle_stability::{run_particle_stability, ParticleStabilityResult};
use crate::qecc::{identify_qecc, QeccIdentification};
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
    let deco_config = deco_config_from_excitation(&config.excitation);
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
    pub independent_dof_per_site: f64,
    pub relative_error: f64,
    pub measured_predicted_agree: bool,
    pub measured_independent_agree: bool,
    pub predicted_independent_agree: bool,
    pub dof_agreement: bool,
    pub stabilizer: StabilizerSearchReport,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

/// Relative / absolute tolerance for U′ two-of-three DOF agreement (tightened vs pre–Phase 9).
const U_PRIME_REL_TOL: f64 = 0.5;
const U_PRIME_ABS_TOL: f64 = 0.35;

pub fn run_eft_dimension_probe(config: &EftDimensionConfig) -> EftDimensionResult {
    let excitation = run_excitation_subspace_probe(&config.excitation);
    let deco_config = deco_config_from_excitation(&config.excitation);
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

    let window_len = window.len().max(1);
    let branch_rank_avg =
        (excitation.branch0_effective_rank + excitation.branch1_effective_rank) / 2.0;
    let measured_dof_per_site = branch_rank_avg / window_len as f64;
    let predicted_dof_per_site = stabilizer.code_rate;
    let independent_dof_per_site = independent_dof_per_site(&branch0, &branch1, &window);
    let relative_error = if predicted_dof_per_site > 1e-6 {
        ((measured_dof_per_site - predicted_dof_per_site) / predicted_dof_per_site).abs()
    } else {
        measured_dof_per_site
    };
    let (dof_agreement, measured_predicted_agree, measured_independent_agree, predicted_independent_agree) =
        two_of_three_dof_agreement(
            measured_dof_per_site,
            predicted_dof_per_site,
            independent_dof_per_site,
            U_PRIME_REL_TOL,
            U_PRIME_ABS_TOL,
        );

    EftDimensionResult {
        window_sites: window_len,
        measured_dof_per_site,
        predicted_dof_per_site,
        independent_dof_per_site,
        relative_error,
        measured_predicted_agree,
        measured_independent_agree,
        predicted_independent_agree,
        dof_agreement,
        stabilizer,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}

/// QECC probe shares the same config shape as the stabilizer search.
pub type QeccProbeConfig = StabilizerSearchConfig;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QeccProbeResult {
    pub excitation: ExcitationSubspaceResult,
    pub stabilizer: StabilizerSearchReport,
    pub qecc: QeccIdentification,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

pub fn run_qecc_probe(config: &QeccProbeConfig) -> QeccProbeResult {
    let excitation = run_excitation_subspace_probe(&config.excitation);
    let deco_config = deco_config_from_excitation(&config.excitation);
    let (psi, n_chain, env_q) = decoherence_final_state(&deco_config);
    let branch0 = chain_conditional_state(&psi, n_chain, env_q, 0);
    let branch1 = chain_conditional_state(&psi, n_chain, env_q, 1);
    let window = excitation.window_sites.clone();
    let stabilizer = search_stabilizers(&psi, n_chain, env_q, &branch0, &branch1, &window);
    let qecc = identify_qecc(&stabilizer, window.len());
    QeccProbeResult {
        excitation,
        stabilizer,
        qecc,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eft_dimension_two_of_three_on_demo_quench() {
        let config = EftDimensionConfig {
            excitation: ExcitationSubspaceConfig {
                n: 8,
                field: 1.2,
                dt: 0.2,
                steps: 22,
                couple_step: 7,
                coupling: 0.9,
                seed: 4242,
                window_radius: 2,
                model: None,
            },
        };
        let r = run_eft_dimension_probe(&config);
        assert!(
            r.dof_agreement,
            "measured={:.3} predicted={:.3} independent={:.3} mp={} mi={} pi={}",
            r.measured_dof_per_site,
            r.predicted_dof_per_site,
            r.independent_dof_per_site,
            r.measured_predicted_agree,
            r.measured_independent_agree,
            r.predicted_independent_agree,
        );
    }
}
