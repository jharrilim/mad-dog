//! Hold-out quench configs for refinement diagnostic tests (Phase 9 / F′ V′ R′ C′).
//!
//! [`RefinementThresholds::default`] and pass bands were hand-tuned on
//! [`REFINEMENT_TUNING_DEFAULT`]. Falsification pass criteria evaluate
//! [`REFINEMENT_HOLDOUT_GRID`] only — configs not used during threshold tuning.

use crate::refinement::RefinementThresholds;
use crate::run_factor_dynamics::run_inplace_split_probe;
use crate::run_refinement::{run_predictive_refinement, AdaptiveRefinementConfig};
use crate::run_rt_quench::{
    run_curvature_proxy_quench, run_rt_quench, CurvatureProxyQuenchConfig, RtQuenchConfig,
};
use crate::tensor_split::InplaceSplitConfig;

/// Quench parameters shared by F′, V′, R′, C′ calibration demos.
#[derive(Clone, Copy, Debug)]
pub struct RefinementQuenchParams {
    pub n: usize,
    pub field: f64,
    pub dt: f64,
    pub steps_rt: usize,
    pub steps_adaptive: usize,
    pub seed: u32,
}

/// Demo config used when tuning [`RefinementThresholds`] (not used for battery pass).
pub const REFINEMENT_TUNING_DEFAULT: RefinementQuenchParams = RefinementQuenchParams {
    n: 10,
    field: 1.5,
    dt: 0.2,
    steps_rt: 16,
    steps_adaptive: 18,
    seed: 7711,
};

/// Hand-tuned thresholds (see `refinement.rs` — pressure 0.42, min_rt_r2 0.82, max_rt_slope_dev 0.35).
/// Documented for audit in `docs/circularity-audit.md`.
#[allow(dead_code)]
pub const REFINEMENT_THRESHOLDS_TUNED: RefinementThresholds = RefinementThresholds {
    pressure: 0.42,
    min_rt_r2: 0.82,
    max_rt_slope_dev: 0.35,
};

#[cfg(test)]
fn assert_tuned_thresholds_match_default() {
    let d = RefinementThresholds::default();
    let t = REFINEMENT_THRESHOLDS_TUNED;
    assert_eq!(t.pressure, d.pressure);
    assert_eq!(t.min_rt_r2, d.min_rt_r2);
    assert_eq!(t.max_rt_slope_dev, d.max_rt_slope_dev);
}

/// Hold-out `(n, field, seed)` grid — distinct from [`REFINEMENT_TUNING_DEFAULT`].
pub const REFINEMENT_HOLDOUT_GRID: &[RefinementHoldoutCase] = &[
    RefinementHoldoutCase {
        n: 10,
        field: 1.2,
        seed: 9001,
    },
    RefinementHoldoutCase {
        n: 8,
        field: 1.8,
        seed: 3333,
    },
    RefinementHoldoutCase {
        n: 10,
        field: 1.0,
        seed: 5555,
    },
];

#[derive(Clone, Copy, Debug)]
pub struct RefinementHoldoutCase {
    pub n: usize,
    pub field: f64,
    pub seed: u32,
}

impl RefinementHoldoutCase {
    pub const DT: f64 = 0.2;
    pub const STEPS_RT: usize = 16;
    pub const STEPS_ADAPTIVE: usize = 18;
    pub const DELTA_N: usize = 2;
    pub const XI: f64 = 1.0;

    pub fn label(self) -> String {
        format!("n{} f{:.1} s{}", self.n, self.field, self.seed)
    }
}

#[derive(Clone, Debug)]
pub struct HoldoutEval {
    pub passed: usize,
    pub total: usize,
    pub labels: Vec<String>,
}

impl HoldoutEval {
    pub fn all_passed(&self) -> bool {
        self.passed == self.total
    }

    pub fn summary(&self) -> String {
        format!(
            "holdOut={}/{} [{}]",
            self.passed,
            self.total,
            self.labels.join(", ")
        )
    }
}

fn eval_holdout<F>(cases: &[RefinementHoldoutCase], mut check: F) -> HoldoutEval
where
    F: FnMut(RefinementHoldoutCase) -> bool,
{
    let mut passed = 0usize;
    let mut labels = Vec::new();
    for case in cases {
        let ok = check(*case);
        if ok {
            passed += 1;
        }
        labels.push(format!("{}:{}", case.label(), if ok { "ok" } else { "fail" }));
    }
    HoldoutEval {
        passed,
        total: cases.len(),
        labels,
    }
}

pub fn r_prime_passes(case: RefinementHoldoutCase) -> bool {
    let r = run_rt_quench(&RtQuenchConfig {
        n: case.n,
        field: case.field,
        dt: RefinementHoldoutCase::DT,
        steps: RefinementHoldoutCase::STEPS_RT,
        seed: case.seed,
    });
    r.structured_deviation
}

pub fn f_prime_passes(case: RefinementHoldoutCase) -> bool {
    let p = run_predictive_refinement(&AdaptiveRefinementConfig {
        n: case.n,
        field: case.field,
        dt: RefinementHoldoutCase::DT,
        steps: RefinementHoldoutCase::STEPS_ADAPTIVE,
        seed: case.seed,
        delta_n: Some(RefinementHoldoutCase::DELTA_N),
    });
    p.lead_time > 0 && p.late_split_recoverable
}

pub fn c_prime_passes(case: RefinementHoldoutCase) -> bool {
    let c = run_curvature_proxy_quench(&CurvatureProxyQuenchConfig {
        n: case.n,
        field: case.field,
        dt: RefinementHoldoutCase::DT,
        steps: RefinementHoldoutCase::STEPS_RT,
        seed: case.seed,
        xi: RefinementHoldoutCase::XI,
    });
    c.internally_consistent
}

pub fn v_prime_passes(case: RefinementHoldoutCase) -> bool {
    let v = run_inplace_split_probe(&InplaceSplitConfig {
        n: case.n,
        field: case.field,
        dt: RefinementHoldoutCase::DT,
        steps: RefinementHoldoutCase::STEPS_ADAPTIVE,
        seed: case.seed,
        delta_n: Some(RefinementHoldoutCase::DELTA_N),
    });
    v.inner
        .split_event
        .as_ref()
        .map(|e| e.in_place_improves)
        .unwrap_or(false)
}

pub fn eval_r_prime_holdout() -> HoldoutEval {
    eval_holdout(REFINEMENT_HOLDOUT_GRID, r_prime_passes)
}

pub fn eval_f_prime_holdout() -> HoldoutEval {
    eval_holdout(REFINEMENT_HOLDOUT_GRID, f_prime_passes)
}

pub fn eval_c_prime_holdout() -> HoldoutEval {
    eval_holdout(REFINEMENT_HOLDOUT_GRID, c_prime_passes)
}

pub fn eval_v_prime_holdout() -> HoldoutEval {
    eval_holdout(REFINEMENT_HOLDOUT_GRID, v_prime_passes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tuned_thresholds_match_default() {
        assert_tuned_thresholds_match_default();
    }

    #[test]
    fn tuning_default_differs_from_holdout_grid() {
        for case in REFINEMENT_HOLDOUT_GRID {
            assert!(
                case.n != REFINEMENT_TUNING_DEFAULT.n
                    || (case.field - REFINEMENT_TUNING_DEFAULT.field).abs() > 1e-9
                    || case.seed != REFINEMENT_TUNING_DEFAULT.seed,
                "hold-out case {} must differ from tuning default on (n, field, seed)",
                case.label()
            );
        }
    }

    #[test]
    fn r_prime_holdout_grid_passes() {
        let eval = eval_r_prime_holdout();
        assert!(
            eval.all_passed(),
            "R′ hold-out failed: {}",
            eval.summary()
        );
    }

    #[test]
    fn f_prime_holdout_grid_passes() {
        let eval = eval_f_prime_holdout();
        assert!(
            eval.all_passed(),
            "F′ hold-out failed: {}",
            eval.summary()
        );
    }

    #[test]
    fn c_prime_holdout_grid_passes() {
        let eval = eval_c_prime_holdout();
        assert!(
            eval.all_passed(),
            "C′ hold-out failed: {}",
            eval.summary()
        );
    }

    #[test]
    fn v_prime_holdout_grid_passes() {
        let eval = eval_v_prime_holdout();
        assert!(
            eval.all_passed(),
            "V′ hold-out failed: {}",
            eval.summary()
        );
    }
}
