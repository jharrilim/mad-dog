//! In-place tensor factor split — embed |ψ⟩ into n+Δ without re-quenching (Phase 7 / S10).

use crate::models::tfim_chain;
use crate::quantum::{evolve_interval, QuantumState};
use crate::refinement::{
    compute_refinement_baselines, measure_refinement_diagnostics, suggest_split_site,
    RefinementDiagnostics, RefinementThresholds,
};
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

/// Embed `state` on `n` qubits into `n + delta` by inserting `delta` |0⟩ qubits after `split_site`.
pub fn embed_state_at_split(state: &QuantumState, split_site: usize, delta: usize) -> QuantumState {
    let n_old = state.n;
    let n_new = n_old + delta;
    let mut out = QuantumState::zero(n_new);
    for old_idx in 0..state.dim {
        let mut new_idx = 0usize;
        for q in 0..n_old {
            let bit = (old_idx >> q) & 1;
            let new_q = if q < split_site { q } else { q + delta };
            new_idx |= bit << new_q;
        }
        out.data[2 * new_idx] = state.data[2 * old_idx];
        out.data[2 * new_idx + 1] = state.data[2 * old_idx + 1];
    }
    out
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InplaceSplitEvent {
    pub trigger_step: usize,
    pub split_site: usize,
    pub pre_n: usize,
    pub post_n: usize,
    pub pre: RefinementDiagnostics,
    pub in_place: RefinementDiagnostics,
    pub rerun: RefinementDiagnostics,
    pub in_place_improves: bool,
    pub matches_rerun: bool,
    pub pressure_delta: f64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InplaceSplitConfig {
    pub n: usize,
    pub field: f64,
    pub dt: f64,
    pub steps: usize,
    pub seed: u32,
    #[serde(default)]
    pub delta_n: Option<usize>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InplaceSplitResult {
    pub n: usize,
    pub delta_n: usize,
    pub field: f64,
    pub dt: f64,
    pub steps: usize,
    pub split_event: Option<InplaceSplitEvent>,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

fn state_at_step(
    n: usize,
    field: f64,
    dt: f64,
    step: usize,
    seed: u32,
) -> QuantumState {
    let model = tfim_chain(n, 1.0, field);
    let mut rng = Rng::new(42);
    let radius = model
        .hamiltonian
        .estimate_spectral_radius(&mut rng, 30)
        .max(1e-6);
    let center = n / 2;
    let mut state = QuantumState::zero(n);
    state.data[2 * (1 << center)] = 1.0;
    for _ in 0..step {
        state = evolve_interval(&model.hamiltonian, &state, dt, radius, 6);
    }
    let _ = seed;
    state
}

fn evolve_from_state(
    state: QuantumState,
    n: usize,
    field: f64,
    dt: f64,
    extra_steps: usize,
) -> QuantumState {
    if extra_steps == 0 {
        return state;
    }
    let model = tfim_chain(n, 1.0, field);
    let mut rng = Rng::new(42);
    let radius = model
        .hamiltonian
        .estimate_spectral_radius(&mut rng, 30)
        .max(1e-6);
    let mut out = state;
    for _ in 0..extra_steps {
        out = evolve_interval(&model.hamiltonian, &out, dt, radius, 6);
    }
    out
}

pub fn run_inplace_split(config: &InplaceSplitConfig) -> InplaceSplitResult {
    let delta_n = config.delta_n.unwrap_or(2);
    let baselines = compute_refinement_baselines(config.n, config.field, config.seed);
    let thresholds = RefinementThresholds::default();

    // Find trigger: peak pressure step on n-chain quench that ends in failure.
    let mut peak_step = 0usize;
    let mut peak_pressure = 0.0_f64;
    let mut peak_state = state_at_step(config.n, config.field, config.dt, 0, config.seed);
    let mut peak_diag = measure_refinement_diagnostics(&peak_state, 1, &baselines, &thresholds);

    for step in 0..=config.steps {
        let state = state_at_step(config.n, config.field, config.dt, step, config.seed);
        let diag = measure_refinement_diagnostics(&state, 1, &baselines, &thresholds);
        if diag.pressure > peak_pressure {
            peak_pressure = diag.pressure;
            peak_step = step;
            peak_state = state;
            peak_diag = diag;
        }
    }

    let split_event = if peak_diag.needs_refinement {
        let split_site = suggest_split_site(&peak_state);
        let pre = peak_diag;
        let remaining = config.steps.saturating_sub(peak_step);

        let embedded = embed_state_at_split(&peak_state, split_site, delta_n);
        let post_n = config.n + delta_n;
        let baselines_large =
            compute_refinement_baselines(post_n, config.field, config.seed);
        let in_place_state = evolve_from_state(embedded, post_n, config.field, config.dt, remaining);
        let in_place = measure_refinement_diagnostics(
            &in_place_state,
            1,
            &baselines_large,
            &thresholds,
        );

        let rerun_state = state_at_step(post_n, config.field, config.dt, config.steps, config.seed);
        let rerun = measure_refinement_diagnostics(
            &rerun_state,
            1,
            &baselines_large,
            &thresholds,
        );

        let in_place_improves = in_place.pressure < pre.pressure - 1e-6;
        let matches_rerun = (in_place.pressure - rerun.pressure).abs() < 0.15;
        let pressure_delta = pre.pressure - in_place.pressure;

        Some(InplaceSplitEvent {
            trigger_step: peak_step,
            split_site,
            pre_n: config.n,
            post_n,
            pre,
            in_place,
            rerun,
            in_place_improves,
            matches_rerun,
            pressure_delta,
        })
    } else {
        None
    };

    InplaceSplitResult {
        n: config.n,
        delta_n,
        field: config.field,
        dt: config.dt,
        steps: config.steps,
        split_event,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embed_preserves_norm_on_product_state() {
        let mut s = QuantumState::zero(4);
        s.data[2 * 5] = 1.0;
        let e = embed_state_at_split(&s, 2, 1);
        assert_eq!(e.n, 5);
        let norm: f64 = e.data.chunks(2).map(|c| c[0] * c[0] + c[1] * c[1]).sum();
        assert!((norm - 1.0).abs() < 1e-9);
    }

    #[test]
    fn inplace_split_improves_diagnostics() {
        let r = run_inplace_split(&InplaceSplitConfig {
            n: 10,
            field: 1.5,
            dt: 0.2,
            steps: 18,
            seed: 7711,
            delta_n: Some(2),
        });
        let ev = r.split_event.expect("split trigger");
        assert!(
            ev.in_place_improves,
            "pre={:.3} in_place={:.3}",
            ev.pre.pressure,
            ev.in_place.pressure
        );
    }
}
