//! QECC code subspace identification — [[n,k,d]] parameters and branch fidelity.
//!
//! Builds on the generators found by [`stabilizer_search`].  Given a set of commuting
//! Pauli generators {g₁,…,gₘ} that sharpen on branch states, the code subspace is their
//! shared eigenspace.  The fidelity of a state with that subspace is:
//!
//!   F(|ψ⟩) = ∏ᵢ (1 + λᵢ ⟨ψ|gᵢ|ψ⟩) / 2
//!
//! where λᵢ = sign(⟨branch|gᵢ|branch⟩) is the eigenvalue the branches approximately
//! occupy.  Using |⟨g⟩| = λᵢ⟨g⟩ (valid because the search ensures both branches agree
//! in sign) simplifies this to:
//!
//!   F(|ψ⟩) ≈ ∏ᵢ (1 + |⟨ψ|gᵢ|ψ⟩|) / 2
//!
//! which equals 1 when |ψ⟩ is exactly in the +1 eigenspace of all generators and is
//! computable directly from the expectations already stored in [`StabilizerGenerator`].
//!
//! **Selectivity** (branch_avg / mixed) >1 means the code subspace is branch-selective —
//! the core QECC signature: IR matter lives in a subspace that Everett branches carve out.

use crate::stabilizer_search::StabilizerSearchReport;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QeccIdentification {
    /// Physical qubits (window size = n of the code).
    pub n_physical: usize,
    /// Logical qubits = n_physical − generator_count.
    pub k_logical: usize,
    /// Minimum logical operator weight (code distance).
    pub d_distance: usize,
    /// Human-readable code label, e.g. "[[5,1,3]]".
    pub code_label: String,
    /// Fidelity of branch 0 with code subspace: ∏_g (1 + |⟨g⟩₀|)/2.
    pub branch0_code_fidelity: f64,
    /// Fidelity of branch 1 with code subspace: ∏_g (1 + |⟨g⟩₁|)/2.
    pub branch1_code_fidelity: f64,
    /// Fidelity of mixed state with code subspace: ∏_g (1 + |⟨g⟩_mixed|)/2.
    pub mixed_code_fidelity: f64,
    /// Average of branch 0 and branch 1 fidelities.
    pub branch_avg_fidelity: f64,
    /// branch_avg / mixed — >1 means branches are more in the code subspace than the
    /// mixed (pre-decohered) state.
    pub fidelity_selectivity: f64,
    /// True if k ≥ 1, stabilizer found, and avg branch fidelity > 0.5.
    pub code_subspace_found: bool,
}

fn state_fidelity(report: &StabilizerSearchReport, get_e: impl Fn(&crate::stabilizer_search::StabilizerGenerator) -> f64) -> f64 {
    if report.generators.is_empty() {
        return 0.0;
    }
    report.generators.iter().fold(1.0, |acc, gen| {
        acc * (1.0 + get_e(gen).abs()) / 2.0
    })
}

pub fn identify_qecc(report: &StabilizerSearchReport, n_physical: usize) -> QeccIdentification {
    let k_logical = n_physical.saturating_sub(report.generator_count);
    let d_distance = report.code_distance;
    let code_label = format!("[[{n_physical},{k_logical},{d_distance}]]");

    let b0 = state_fidelity(report, |g| g.branch0_expectation);
    let b1 = state_fidelity(report, |g| g.branch1_expectation);
    let mixed = state_fidelity(report, |g| g.mixed_expectation);
    let branch_avg = (b0 + b1) / 2.0;
    let fidelity_selectivity = if mixed > 1e-9 {
        (branch_avg / mixed).min(99.0)
    } else if branch_avg > 1e-9 {
        99.0
    } else {
        1.0
    };
    let code_subspace_found = k_logical >= 1 && report.stabilizer_found && branch_avg > 0.5;

    QeccIdentification {
        n_physical,
        k_logical,
        d_distance,
        code_label,
        branch0_code_fidelity: b0,
        branch1_code_fidelity: b1,
        mixed_code_fidelity: mixed,
        branch_avg_fidelity: branch_avg,
        fidelity_selectivity,
        code_subspace_found,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoherence::{chain_conditional_state, decoherence_final_state, DecoherenceQuenchConfig};
    use crate::excitation_subspace::{run_excitation_subspace_probe, ExcitationSubspaceConfig};
    use crate::stabilizer_search::search_stabilizers;

    fn demo_qecc() -> QeccIdentification {
        let config = ExcitationSubspaceConfig {
            n: 8,
            field: 1.2,
            dt: 0.2,
            steps: 22,
            couple_step: 7,
            coupling: 0.9,
            seed: 4242,
            window_radius: 2,
        };
        let probe = run_excitation_subspace_probe(&config);
        let deco = DecoherenceQuenchConfig {
            n: config.n,
            field: config.field,
            dt: config.dt,
            steps: config.steps,
            couple_step: config.couple_step,
            coupling: config.coupling,
            seed: config.seed,
        };
        let (psi, n_chain, env_q) = decoherence_final_state(&deco);
        let b0 = chain_conditional_state(&psi, n_chain, env_q, 0);
        let b1 = chain_conditional_state(&psi, n_chain, env_q, 1);
        let report = search_stabilizers(&psi, n_chain, env_q, &b0, &b1, &probe.window_sites);
        identify_qecc(&report, probe.window_sites.len())
    }

    #[test]
    fn code_subspace_found_on_demo_quench() {
        let q = demo_qecc();
        assert!(
            q.code_subspace_found,
            "k={} d={} b0={:.3} b1={:.3} avg={:.3}",
            q.k_logical, q.d_distance, q.branch0_code_fidelity, q.branch1_code_fidelity, q.branch_avg_fidelity
        );
    }

    #[test]
    fn branch_fidelity_exceeds_mixed_on_demo_quench() {
        let q = demo_qecc();
        assert!(
            q.fidelity_selectivity > 1.05,
            "selectivity={:.3} branch_avg={:.3} mixed={:.3}",
            q.fidelity_selectivity, q.branch_avg_fidelity, q.mixed_code_fidelity
        );
    }

    #[test]
    fn code_label_format() {
        let q = demo_qecc();
        assert!(q.code_label.starts_with("[["), "label: {}", q.code_label);
    }
}
