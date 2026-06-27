//! Hold-out grids for T′ (Born weights) and I′ (stabilizer search) — Phase 9 Priority 4.
//!
//! T′ calibration: `n=8, field=1.2, dt=0.2, steps=22, couple_step=7` (demo quench in
//! run_falsification.rs).  Pass bands are baked into [`branch_born::run_branch_born_study`].
//!
//! I′ calibration: `coupling=0.9, window_radius=2` (same quench otherwise).  Pass bands are
//! baked into [`stabilizer_search::branch_stabilizer_candidate`].
//!
//! Battery pass criteria evaluate their respective [`BORN_HOLDOUT_GRID`] /
//! [`STAB_HOLDOUT_GRID`] only — configs not used during threshold tuning.

use crate::branch_born::run_branch_born_study;
use crate::excitation_subspace::ExcitationSubspaceConfig;
use crate::refinement_holdout::HoldoutEval;
use crate::run_matter::{run_stabilizer_search, StabilizerSearchConfig};

// ─── T′ — Born hold-out ───────────────────────────────────────────────────────

/// Outer quench parameters swept for T′ hold-out.  Coupling is swept *internally* by
/// [`branch_born::run_branch_born_study`] so it is not a free variable here.
#[derive(Clone, Copy, Debug)]
pub struct BornHoldoutCase {
    pub n: usize,
    pub field: f64,
    pub couple_step: usize,
}

impl BornHoldoutCase {
    pub const DT: f64 = 0.2;
    pub const STEPS: usize = 22;

    pub fn label(self) -> String {
        format!("n{} f{:.1} cs{}", self.n, self.field, self.couple_step)
    }
}

/// Demo config used when `born_consistent` thresholds were set (not used for battery pass).
pub const BORN_TUNING_DEFAULT: BornHoldoutCase =
    BornHoldoutCase { n: 8, field: 1.2, couple_step: 7 };

/// Hold-out `(n, field, couple_step)` grid — distinct from [`BORN_TUNING_DEFAULT`].
pub const BORN_HOLDOUT_GRID: &[BornHoldoutCase] = &[
    BornHoldoutCase { n: 10, field: 1.0, couple_step: 8 },
    BornHoldoutCase { n: 8,  field: 1.4, couple_step: 6 },
    BornHoldoutCase { n: 10, field: 1.6, couple_step: 7 },
];

pub fn t_prime_passes(case: BornHoldoutCase) -> bool {
    let r = run_branch_born_study(
        case.n,
        case.field,
        BornHoldoutCase::DT,
        BornHoldoutCase::STEPS,
        case.couple_step,
    );
    r.born_consistent
}

pub fn eval_t_prime_holdout() -> HoldoutEval {
    let mut passed = 0usize;
    let mut labels = Vec::new();
    for &case in BORN_HOLDOUT_GRID {
        let ok = t_prime_passes(case);
        if ok {
            passed += 1;
        }
        labels.push(format!("{}:{}", case.label(), if ok { "ok" } else { "fail" }));
    }
    HoldoutEval { passed, total: BORN_HOLDOUT_GRID.len(), labels }
}

// ─── I′ — Stabilizer hold-out ─────────────────────────────────────────────────

/// Coupling / window-radius pairs swept for I′ hold-out.  All other quench params are
/// fixed at the calibration demo values.
#[derive(Clone, Copy, Debug)]
pub struct StabHoldoutCase {
    pub coupling: f64,
    pub window_radius: usize,
}

impl StabHoldoutCase {
    // Fixed outer quench params (same as calibration demo)
    pub const N: usize = 8;
    pub const FIELD: f64 = 1.2;
    pub const DT: f64 = 0.2;
    pub const STEPS: usize = 22;
    pub const COUPLE_STEP: usize = 7;
    pub const SEED: u32 = 4242;

    pub fn label(self) -> String {
        format!("c{:.2} wr{}", self.coupling, self.window_radius)
    }
}

/// Demo config used when stabilizer pass bands were calibrated (not used for battery pass).
pub const STAB_TUNING_DEFAULT: StabHoldoutCase =
    StabHoldoutCase { coupling: 0.9, window_radius: 2 };

/// Hold-out `(coupling, window_radius)` grid — distinct from [`STAB_TUNING_DEFAULT`].
pub const STAB_HOLDOUT_GRID: &[StabHoldoutCase] = &[
    StabHoldoutCase { coupling: 0.7, window_radius: 2 },
    StabHoldoutCase { coupling: 0.9, window_radius: 3 },
    StabHoldoutCase { coupling: 0.85, window_radius: 2 },
];

pub fn i_prime_passes(case: StabHoldoutCase) -> bool {
    let s = run_stabilizer_search(&StabilizerSearchConfig {
        excitation: ExcitationSubspaceConfig {
            n: StabHoldoutCase::N,
            field: StabHoldoutCase::FIELD,
            dt: StabHoldoutCase::DT,
            steps: StabHoldoutCase::STEPS,
            couple_step: StabHoldoutCase::COUPLE_STEP,
            coupling: case.coupling,
            seed: StabHoldoutCase::SEED,
            window_radius: case.window_radius,
            model: None,
        },
    });
    s.stabilizer.stabilizer_found && s.distance_scales
}

pub fn eval_i_prime_holdout() -> HoldoutEval {
    let mut passed = 0usize;
    let mut labels = Vec::new();
    for &case in STAB_HOLDOUT_GRID {
        let ok = i_prime_passes(case);
        if ok {
            passed += 1;
        }
        labels.push(format!("{}:{}", case.label(), if ok { "ok" } else { "fail" }));
    }
    HoldoutEval { passed, total: STAB_HOLDOUT_GRID.len(), labels }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn born_holdout_differs_from_tuning_default() {
        for case in BORN_HOLDOUT_GRID {
            assert!(
                case.n != BORN_TUNING_DEFAULT.n
                    || (case.field - BORN_TUNING_DEFAULT.field).abs() > 1e-9
                    || case.couple_step != BORN_TUNING_DEFAULT.couple_step,
                "Born hold-out case {} must differ from tuning default on (n, field, couple_step)",
                case.label()
            );
        }
    }

    #[test]
    fn stab_holdout_differs_from_tuning_default() {
        for case in STAB_HOLDOUT_GRID {
            assert!(
                (case.coupling - STAB_TUNING_DEFAULT.coupling).abs() > 1e-9
                    || case.window_radius != STAB_TUNING_DEFAULT.window_radius,
                "Stab hold-out case {} must differ from tuning default on (coupling, window_radius)",
                case.label()
            );
        }
    }

    #[test]
    fn t_prime_holdout_grid_passes() {
        let eval = eval_t_prime_holdout();
        assert!(eval.all_passed(), "T′ hold-out failed: {}", eval.summary());
    }

    #[test]
    fn i_prime_holdout_grid_passes() {
        let eval = eval_i_prime_holdout();
        assert!(eval.all_passed(), "I′ hold-out failed: {}", eval.summary());
    }
}

