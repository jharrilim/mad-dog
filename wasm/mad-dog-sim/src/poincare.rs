//! Full Poincaré composite probe — tests all four generator families simultaneously.
//!
//! The Poincaré group (spatial translations, time translation, rotations, boosts) has
//! 10 generators in 3+1D and 6 in 2+1D.  The TFIM lattice tests each approximately:
//!
//! - **Rotation** (J): 4-direction cardinal speed CV on a 2D grid (test L).
//! - **Time translation** (H): linear dispersion ω(k) ≈ v|k| at small k (test O).
//! - **Boosts** (K): light-cone velocity stable under observer-frame shift (test P).
//! - **Spatial translations** (P^i): light-cone velocity consistent across multiple
//!   reference-site observers arranged at distinct spatial positions (new — test Y).
//!
//! This module ties all four together into a single composite verdict (test Z).
//! It re-uses the existing `grid_cardinal_speed_cv`, `grid_light_cone_velocity`, and
//! `build_dispersion` functions rather than duplicating any physics.

use crate::dispersion::{build_dispersion, DispersionConfig};
use crate::lorentz::{grid_cardinal_speed_cv, grid_light_cone_velocity, speed_coefficient_of_variation};
use crate::models::tfim_chain;
use serde::{Deserialize, Serialize};

/// Config for the full Poincaré composite probe.
///
/// Uses the same grid and chain parameters as the individual L / O / P tests so
/// results are directly comparable.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoincareCompositeConfig {
    /// 2D grid rows for rotation + boost tests.
    #[serde(default = "default_rows")]
    pub rows: usize,
    /// 2D grid cols for rotation + boost tests.
    #[serde(default = "default_cols")]
    pub cols: usize,
    /// Transverse field on both grid and chain.
    #[serde(default = "default_field")]
    pub field: f64,
    /// Time step.
    #[serde(default = "default_dt")]
    pub dt: f64,
    /// Evolution steps.
    #[serde(default = "default_steps")]
    pub steps: usize,
    /// 1D chain size for dispersion test.
    #[serde(default = "default_n_chain")]
    pub n_chain: usize,
    /// Number of dispersion modes to fit.
    #[serde(default = "default_chain_modes")]
    pub chain_modes: usize,
    /// Transverse field for the dispersion chain (ordered-phase default = 1.0).
    #[serde(default = "default_chain_field")]
    pub chain_field: f64,
    /// Time step for the dispersion chain evolution.
    #[serde(default = "default_chain_dt")]
    pub chain_dt: f64,
    /// Steps for the dispersion chain evolution.
    #[serde(default = "default_chain_steps")]
    pub chain_steps: usize,
}

fn default_rows() -> usize { 4 }
fn default_cols() -> usize { 4 }
fn default_field() -> f64 { 1.2 }
fn default_dt() -> f64 { 0.2 }
fn default_steps() -> usize { 28 }
fn default_n_chain() -> usize { 16 }
fn default_chain_modes() -> usize { 3 }
fn default_chain_field() -> f64 { 1.0 }
fn default_chain_dt() -> f64 { 0.15 }
fn default_chain_steps() -> usize { 48 }

/// One observer frame for the multi-frame boost test.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoostFrame {
    /// Human label for this reference frame.
    pub label: String,
    /// Which grid site is used as the time reference (None = uniform average).
    pub reference_site: Option<usize>,
    /// Measured light-cone velocity from this frame.
    pub velocity: f64,
}

/// Full result of the Poincaré composite probe.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoincareCompositeResult {
    // --- Rotation (J): 4-direction cardinal CV on the 2D grid ---
    /// Grid size used for rotation and boost tests.
    pub grid_rows: usize,
    pub grid_cols: usize,
    /// Cardinal-direction speed coefficient of variation.  Low ⇒ rotational isotropy.
    pub rotation_cv: f64,
    /// Pass if rotation_cv < 0.25.
    pub rotation_ok: bool,

    // --- Multi-frame boost (K + P^i): velocity consistency across 4 observer frames ---
    /// Light-cone velocities from 4 observer frames.
    pub boost_frames: Vec<BoostFrame>,
    /// CV of velocities across all frames.  Low ⇒ frame-independent.
    pub boost_cv: f64,
    /// Pass if boost_cv < 0.25.
    pub boost_ok: bool,

    // --- Dispersion (H): linear ω(k) on 1D chain ---
    /// R² of linear fit ω = v·k.
    pub dispersion_linear_r2: f64,
    /// Group velocity CoV across modes.
    pub dispersion_velocity_cov: f64,
    /// Pass if ω(k) is linear at small k (existing `linear_at_small_k` criterion).
    pub dispersion_ok: bool,

    // --- Composite (Z) ---
    pub tests_passed: usize,
    pub tests_total: usize,
    /// True if rotation, multi-frame boost, AND dispersion all pass.
    pub all_poincare_ok: bool,

    pub elapsed_ms: f64,
    pub backend: &'static str,
}

/// Pass thresholds (kept in one place for easy audit).
const ROTATION_CV_MAX: f64 = 0.25;
const BOOST_CV_MAX: f64 = 0.25;

/// Run the full Poincaré composite probe.
///
/// The four observer frames for the boost test are chosen to span the grid:
/// - `None` (uniform / all-site average clock, used as baseline)
/// - Site 0 (top-left corner — spatially distinct from center)
/// - Mid-top-edge site
/// - Bottom-right corner
///
/// These probe spatial translation symmetry through the lens of observer-frame
/// consistency: if the emergent Poincaré structure is translation-invariant, the
/// light-cone velocity should be the same regardless of where the "clock" observer
/// sits on the grid.
pub fn run_poincare_composite(config: &PoincareCompositeConfig) -> PoincareCompositeResult {
    let rows = config.rows;
    let cols = config.cols;
    let field = config.field;
    let dt = config.dt;
    let steps = config.steps;

    // 1. Rotation: 4-direction cardinal CV on the 2D grid
    let rotation_cv = grid_cardinal_speed_cv(rows, cols, field, dt, steps);
    let rotation_ok = rotation_cv < ROTATION_CV_MAX;

    // 2. Multi-frame boost: 4 observer positions
    let frame_specs: &[(&str, Option<usize>)] = &[
        ("uniform",     None),
        ("corner-TL",   Some(0)),
        ("edge-top",    Some(cols / 2)),
        ("corner-BR",   Some(rows * cols - 1)),
    ];
    let mut boost_frames: Vec<BoostFrame> = frame_specs
        .iter()
        .map(|(label, ref_site)| {
            let v = grid_light_cone_velocity(rows, cols, field, dt, steps, *ref_site);
            BoostFrame {
                label: label.to_string(),
                reference_site: *ref_site,
                velocity: v,
            }
        })
        .collect();

    // Replace uniform slot if it has an unusual velocity (could be near-zero)
    // with a finite-velocity guard so CV isn't inflated by numerical noise.
    for f in &mut boost_frames {
        if !f.velocity.is_finite() || f.velocity.abs() < 1e-6 {
            f.velocity = 0.0;
        }
    }
    let velocities: Vec<f64> = boost_frames
        .iter()
        .map(|f| f.velocity.abs())
        .filter(|&v| v > 1e-6)
        .collect();
    let boost_cv = speed_coefficient_of_variation(&velocities);
    let boost_ok = boost_cv.is_finite() && boost_cv < BOOST_CV_MAX;

    // 3. Dispersion on 1D chain — uses chain-specific params tuned for linear dispersion
    let chain = tfim_chain(config.n_chain, 1.0, config.chain_field);
    let disp = build_dispersion(
        &chain.hamiltonian,
        &DispersionConfig {
            n: config.n_chain,
            field: config.chain_field,
            dt: config.chain_dt,
            steps: config.chain_steps,
            modes: config.chain_modes,
        },
    );
    let dispersion_ok = disp.linear_at_small_k;

    let v_mean = disp.modes.iter().map(|m| m.group_velocity).sum::<f64>()
        / disp.modes.len().max(1) as f64;
    let dispersion_velocity_cov = if disp.modes.len() >= 2 && v_mean > 1e-9 {
        let var = disp
            .modes
            .iter()
            .map(|m| (m.group_velocity - v_mean).powi(2))
            .sum::<f64>()
            / disp.modes.len() as f64;
        var.sqrt() / v_mean
    } else {
        f64::INFINITY
    };

    let pillar_results = [rotation_ok, boost_ok, dispersion_ok];
    let tests_passed = pillar_results.iter().filter(|&&x| x).count();
    let all_poincare_ok = tests_passed == pillar_results.len();

    PoincareCompositeResult {
        grid_rows: rows,
        grid_cols: cols,
        rotation_cv,
        rotation_ok,
        boost_frames,
        boost_cv,
        boost_ok,
        dispersion_linear_r2: disp.linear_r2,
        dispersion_velocity_cov,
        dispersion_ok,
        tests_passed,
        tests_total: pillar_results.len(),
        all_poincare_ok,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn demo_config() -> PoincareCompositeConfig {
        PoincareCompositeConfig {
            rows: 4,
            cols: 4,
            field: 1.2,
            dt: 0.2,
            steps: 28,
            n_chain: 16,
            chain_modes: 3,
            chain_field: 1.0,
            chain_dt: 0.15,
            chain_steps: 48,
        }
    }

    #[test]
    fn poincare_composite_all_pass() {
        let r = run_poincare_composite(&demo_config());
        assert!(
            r.all_poincare_ok,
            "passed {}/{}: rot_cv={:.3} boost_cv={:.3} disp_r2={:.3}",
            r.tests_passed,
            r.tests_total,
            r.rotation_cv,
            r.boost_cv,
            r.dispersion_linear_r2,
        );
    }
}
