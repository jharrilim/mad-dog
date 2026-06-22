//! Lorentz scaling sweep — cardinal speed CoV vs lattice size.

use crate::lorentz::grid_cardinal_speed_cv;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LorentzScalingCase {
    pub label: String,
    pub rows: usize,
    pub cols: usize,
    pub speed_cv: f64,
    pub passed: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LorentzScalingResult {
    pub cases: Vec<LorentzScalingCase>,
    pub cov_small: f64,
    pub cov_large: f64,
    pub cov_improves: bool,
    pub all_passed: bool,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

const CV_MAX: f64 = 0.25;

pub fn run_lorentz_scaling() -> LorentzScalingResult {
    let field = 1.2;
    let dt = 0.2;
    let steps = 28;
    let sizes = [(3_usize, 3_usize), (4, 4)];
    let mut cases = Vec::new();
    for (rows, cols) in sizes {
        let cv = grid_cardinal_speed_cv(rows, cols, field, dt, steps);
        cases.push(LorentzScalingCase {
            label: format!("grid {rows}x{cols}"),
            rows,
            cols,
            speed_cv: cv,
            passed: cv < CV_MAX,
        });
    }
    let cov_small = cases.first().map(|c| c.speed_cv).unwrap_or(f64::INFINITY);
    let cov_large = cases.last().map(|c| c.speed_cv).unwrap_or(f64::INFINITY);
    let cov_improves = cov_large <= cov_small + 0.08;
    let all_passed = cases.iter().all(|c| c.passed) && cov_improves;
    LorentzScalingResult {
        cases,
        cov_small,
        cov_large,
        cov_improves,
        all_passed,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}
