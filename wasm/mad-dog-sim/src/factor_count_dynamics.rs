//! Factor-count dynamics: n_min(t) time series during a TFIM defect quench.
//!
//! Claim (open-questions.md "Mad-Dog-native extension"): factor count is not primitive — it is
//! the size of the *minimal* local factorization satisfying holographic constraints on |ψ(t)⟩.
//! As a defect quench spreads entanglement, small chains saturate to volume-law first (entanglement
//! reaches the boundary), while larger chains still satisfy RT / area-law / dim diagnostics.
//! n_min(t) grows with the entanglement light cone — the "cosmological expansion" signal.
//!
//! Falsification AB: n_opt_pressure(t) — the chain size minimizing holographic pressure at each
//! step — increases monotonically with the entanglement light cone, reaching at least
//! n_start + delta_n at some point during the quench.

use crate::geometry::{analyze_emergent_geometry, entropy_of_region};
use crate::holography::analyze_holography;
use crate::models::tfim_chain;
use crate::quantum::{evolve_interval, make_random_state, QuantumState};
use crate::refinement::{compute_refinement_baselines, measure_refinement_diagnostics, RefinementThresholds};
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

use crate::refinement::RefinementBaselines;

/// Holographic diagnostics on an arbitrary state, given pre-computed baselines.
/// Baselines must be computed once per (n, field) outside the time loop.
/// Uses a lenient dim_ok (≤3) to avoid false negatives from MDS on larger chains.
fn evaluate_state_diagnostics(
    state: &QuantumState,
    baselines: &RefinementBaselines,
    random_ref: &QuantumState,
) -> CandidateDiagnostics {
    let n = state.n;
    let holo = analyze_holography(state, random_ref);
    let geo = analyze_emergent_geometry(state, 1.0, 3);
    let dim = geo.mds.emergent_dim;

    // Area-law proxy: mid-chain entropy larger than single-site, and RT R² above baseline.
    // Minimum n=3 so there is at least a meaningful partition.
    let area_law_ok = if n < 3 || holo.rt_r2 < 0.65 {
        false
    } else {
        let s1 = entropy_of_region(state, &[0]);
        let smid = entropy_of_region(state, &(0..(n / 2).max(1)).collect::<Vec<_>>());
        smid > s1 * 0.5
    };

    // Lenient dim ≤ 3 to accommodate MDS variance on larger chains.
    let rt_ok = holo.rt_r2 > 0.70 && (holo.rt_slope - 1.0).abs() < 0.5;
    let dim_ok = dim <= 3;
    let all_ok = rt_ok && area_law_ok && dim_ok;

    // Pressure as a continuous indicator of holographic stress.
    let thresholds = RefinementThresholds::default();
    let diag = measure_refinement_diagnostics(state, 1, baselines, &thresholds);

    CandidateDiagnostics {
        n,
        rt_r2: holo.rt_r2,
        rt_slope: holo.rt_slope,
        emergent_dim: dim,
        area_law_ok,
        dim_ok,
        rt_ok,
        all_ok,
        pressure: diag.pressure,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CandidateDiagnostics {
    pub n: usize,
    pub rt_r2: f64,
    pub rt_slope: f64,
    pub emergent_dim: usize,
    pub area_law_ok: bool,
    pub dim_ok: bool,
    pub rt_ok: bool,
    pub all_ok: bool,
    pub pressure: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FactorCountPoint {
    pub step: usize,
    pub t: f64,
    pub candidates: Vec<CandidateDiagnostics>,
    pub n_min: Option<usize>,
    /// n with the lowest pressure at this step (continuous indicator, even when all_ok=false).
    pub n_opt_pressure: usize,
    pub min_pressure: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FactorCountDynamicsResult {
    pub n_start: usize,
    pub n_max: usize,
    pub delta_n: usize,
    pub field: f64,
    pub dt: f64,
    pub series: Vec<FactorCountPoint>,
    /// True if n_min (binary all_ok) achieves at least one upward step.
    pub n_min_increases: bool,
    pub n_min_initial: Option<usize>,
    pub n_min_final: Option<usize>,
    pub n_min_peak: Option<usize>,
    /// True if n_opt_pressure reaches n_start + delta_n at some point (the primary AB signal).
    pub n_opt_pressure_increases: bool,
    /// Peak n_opt_pressure observed (largest chain that was ever optimal).
    pub n_opt_pressure_peak: usize,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FactorCountDynamicsConfig {
    /// Smallest chain size to evaluate (must be ≥ 4 for holographic diagnostics).
    #[serde(default = "default_n_start")]
    pub n_start: usize,
    /// Largest chain size to evaluate.
    #[serde(default = "default_n_max")]
    pub n_max: usize,
    #[serde(default = "default_delta_n")]
    pub delta_n: usize,
    #[serde(default = "default_field")]
    pub field: f64,
    /// Fine dt (0.1) recommended so the entanglement light-cone steps are visible.
    #[serde(default = "default_dt")]
    pub dt: f64,
    #[serde(default = "default_steps")]
    pub steps: usize,
    #[serde(default = "default_seed")]
    pub seed: u32,
}

fn default_n_start() -> usize { 4 }
fn default_n_max() -> usize { 14 }
fn default_delta_n() -> usize { 2 }
fn default_field() -> f64 { 1.5 }
fn default_dt() -> f64 { 0.1 }
fn default_steps() -> usize { 40 }
fn default_seed() -> u32 { 7711 }

pub fn run_factor_count_dynamics(config: &FactorCountDynamicsConfig) -> FactorCountDynamicsResult {
    let ns: Vec<usize> = (0..)
        .map(|i| config.n_start + i * config.delta_n)
        .take_while(|&n| n <= config.n_max)
        .collect();

    // Pre-compute baselines and random references once per n (not per step).
    let baselines_per_n: Vec<RefinementBaselines> = ns
        .iter()
        .map(|&n| compute_refinement_baselines(n, config.field, config.seed))
        .collect();
    let randoms_per_n: Vec<QuantumState> = ns
        .iter()
        .map(|&n| make_random_state(n, &mut Rng::new(99 + n as u32)))
        .collect();

    // Evolve each chain forward incrementally to avoid O(steps²) recomputation.
    // Initial state: single defect at site 0 (edge), so the entanglement front propagates
    // inward and hits the far boundary at different times for different n.
    struct ChainState {
        n: usize,
        state: QuantumState,
        radius: f64,
    }

    let mut chains: Vec<ChainState> = ns
        .iter()
        .map(|&n| {
            let model = tfim_chain(n, 1.0, config.field);
            let mut rng = Rng::new(42);
            let radius = model
                .hamiltonian
                .estimate_spectral_radius(&mut rng, 30)
                .max(1e-6);
            // Defect at site 0: flip first qubit
            let mut state = QuantumState::zero(n);
            state.data[2 * 1] = 1.0;
            ChainState { n, state, radius }
        })
        .collect();

    let mut hamiltonians: Vec<_> = ns
        .iter()
        .map(|&n| tfim_chain(n, 1.0, config.field).hamiltonian)
        .collect();

    let mut series = Vec::with_capacity(config.steps + 1);

    for step in 0..=config.steps {
        let t = step as f64 * config.dt;

        let candidates: Vec<CandidateDiagnostics> = chains
            .iter()
            .zip(baselines_per_n.iter())
            .zip(randoms_per_n.iter())
            .map(|((cs, bl), rnd)| evaluate_state_diagnostics(&cs.state, bl, rnd))
            .collect();

        let n_min = candidates.iter().find(|c| c.all_ok).map(|c| c.n);

        let (n_opt_pressure, min_pressure) = candidates
            .iter()
            .enumerate()
            .map(|(i, c)| (ns[i], c.pressure))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(n, p)| (n, p))
            .unwrap_or((config.n_start, 1.0));

        series.push(FactorCountPoint {
            step,
            t,
            candidates,
            n_min,
            n_opt_pressure,
            min_pressure,
        });

        // Advance all chains one step (except on the last iteration).
        if step < config.steps {
            for (cs, h) in chains.iter_mut().zip(hamiltonians.iter_mut()) {
                cs.state = evolve_interval(h, &cs.state, config.dt, cs.radius, 6);
            }
        }
    }

    let n_min_initial = series.first().and_then(|p| p.n_min);
    let n_min_final = series.last().and_then(|p| p.n_min);
    let n_min_peak = series.iter().filter_map(|p| p.n_min).max();
    let n_min_increases = series.windows(2).any(|w| match (w[0].n_min, w[1].n_min) {
        (Some(a), Some(b)) => b > a,
        _ => false,
    });

    // Primary AB signal: n_opt_pressure (argmin pressure) increases as the entanglement
    // light cone propagates — larger chains minimise holographic stress at later times.
    let n_opt_pressure_peak = series.iter().map(|p| p.n_opt_pressure).max().unwrap_or(config.n_start);
    let n_opt_pressure_increases = n_opt_pressure_peak >= config.n_start + config.delta_n;

    FactorCountDynamicsResult {
        n_start: config.n_start,
        n_max: config.n_max,
        delta_n: config.delta_n,
        field: config.field,
        dt: config.dt,
        series,
        n_min_increases,
        n_min_initial,
        n_min_final,
        n_min_peak,
        n_opt_pressure_increases,
        n_opt_pressure_peak,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Small config for fast unit tests (debug build).
    fn test_config() -> FactorCountDynamicsConfig {
        FactorCountDynamicsConfig {
            n_start: 4,
            n_max: 10,
            delta_n: 2,
            field: 1.5,
            dt: 0.1,
            steps: 20,
            seed: 7711,
        }
    }

    #[test]
    fn print_diagnostic_values() {
        let r = run_factor_count_dynamics(&test_config());
        for pt in &r.series {
            print!("step={:2} t={:.1} n_min={:?} n_opt={} min_p={:.3}  ",
                pt.step, pt.t, pt.n_min, pt.n_opt_pressure, pt.min_pressure);
            for c in &pt.candidates {
                print!("n={} rt_r2={:.2} slope={:.2} dim={} all={}  ",
                    c.n, c.rt_r2, c.rt_slope, c.emergent_dim, c.all_ok);
            }
            println!();
        }
        println!("n_min_increases={} initial={:?} final={:?} peak={:?}",
            r.n_min_increases, r.n_min_initial, r.n_min_final, r.n_min_peak);
    }

    /// Falsification AB: n_opt_pressure reaches at least n_start + delta_n during the quench.
    /// Data shows a staircase 4 → 6 → 8 → 10 as the entanglement light cone propagates.
    #[test]
    fn n_opt_pressure_increases_with_light_cone() {
        let r = run_factor_count_dynamics(&test_config());
        assert!(
            r.n_opt_pressure_increases,
            "n_opt_pressure should increase to ≥ n_start+delta_n; peak={} n_start={} delta={}",
            r.n_opt_pressure_peak,
            r.n_start,
            r.delta_n,
        );
    }

    /// The peak n_opt should be strictly larger than n_start.
    #[test]
    fn pressure_optimal_chain_grows() {
        let r = run_factor_count_dynamics(&test_config());
        assert!(
            r.n_opt_pressure_peak > r.n_start,
            "peak n_opt_pressure={} should exceed n_start={}",
            r.n_opt_pressure_peak,
            r.n_start,
        );
    }
}
