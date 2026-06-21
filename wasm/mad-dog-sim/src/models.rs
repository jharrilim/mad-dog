//! Hamiltonian model builders — port of `src/sim/models.ts`.

use crate::quantum::{Hamiltonian, PauliLetter, PauliOp, PauliTerm};
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TruePosition {
    pub x: f64,
    pub y: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub z: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct LatticeLayout {
    pub true_positions: Vec<TruePosition>,
    pub expected_dim: usize,
}

#[derive(Clone, Debug)]
pub struct BuiltModel {
    pub hamiltonian: Hamiltonian,
    pub layout: LatticeLayout,
    pub label: String,
}

pub fn tfim_chain(n: usize, j: f64, h: f64) -> BuiltModel {
    let mut terms = Vec::new();
    for i in 0..(n - 1) {
        terms.push(PauliTerm {
            coeff: -j,
            ops: vec![
                PauliOp {
                    qubit: i,
                    letter: PauliLetter::Z,
                },
                PauliOp {
                    qubit: i + 1,
                    letter: PauliLetter::Z,
                },
            ],
        });
    }
    for i in 0..n {
        terms.push(PauliTerm {
            coeff: -h,
            ops: vec![PauliOp {
                qubit: i,
                letter: PauliLetter::X,
            }],
        });
    }
    let true_positions = (0..n)
        .map(|i| TruePosition {
            x: i as f64,
            y: 0.0,
            z: None,
        })
        .collect();
    BuiltModel {
        hamiltonian: Hamiltonian::new(n, terms),
        layout: LatticeLayout {
            true_positions,
            expected_dim: 1,
        },
        label: format!("TFIM chain (n={n})"),
    }
}

pub fn tfim_grid(rows: usize, cols: usize, j: f64, h: f64) -> BuiltModel {
    let n = rows * cols;
    let idx = |r: usize, c: usize| r * cols + c;
    let mut terms = Vec::new();
    for r in 0..rows {
        for c in 0..cols {
            if c + 1 < cols {
                terms.push(PauliTerm {
                    coeff: -j,
                    ops: vec![
                        PauliOp {
                            qubit: idx(r, c),
                            letter: PauliLetter::Z,
                        },
                        PauliOp {
                            qubit: idx(r, c + 1),
                            letter: PauliLetter::Z,
                        },
                    ],
                });
            }
            if r + 1 < rows {
                terms.push(PauliTerm {
                    coeff: -j,
                    ops: vec![
                        PauliOp {
                            qubit: idx(r, c),
                            letter: PauliLetter::Z,
                        },
                        PauliOp {
                            qubit: idx(r + 1, c),
                            letter: PauliLetter::Z,
                        },
                    ],
                });
            }
        }
    }
    for i in 0..n {
        terms.push(PauliTerm {
            coeff: -h,
            ops: vec![PauliOp {
                qubit: i,
                letter: PauliLetter::X,
            }],
        });
    }
    let mut true_positions = Vec::new();
    for r in 0..rows {
        for c in 0..cols {
            true_positions.push(TruePosition {
                x: c as f64,
                y: r as f64,
                z: None,
            });
        }
    }
    BuiltModel {
        hamiltonian: Hamiltonian::new(n, terms),
        layout: LatticeLayout {
            true_positions,
            expected_dim: 2,
        },
        label: format!("TFIM grid ({rows}x{cols})"),
    }
}

pub fn random_nonlocal(n: usize, rng: &mut Rng) -> BuiltModel {
    let letters = [PauliLetter::X, PauliLetter::Y, PauliLetter::Z];
    let mut terms = Vec::new();
    for i in 0..n {
        for k in (i + 1)..n {
            let pa = letters[(rng.next() * 3.0).floor() as usize % 3];
            let pb = letters[(rng.next() * 3.0).floor() as usize % 3];
            terms.push(PauliTerm {
                coeff: rng.next() * 2.0 - 1.0,
                ops: vec![
                    PauliOp {
                        qubit: i,
                        letter: pa,
                    },
                    PauliOp {
                        qubit: k,
                        letter: pb,
                    },
                ],
            });
        }
        terms.push(PauliTerm {
            coeff: rng.next() * 2.0 - 1.0,
            ops: vec![PauliOp {
                qubit: i,
                letter: letters[(rng.next() * 3.0).floor() as usize % 3],
            }],
        });
    }
    let true_positions = (0..n)
        .map(|i| TruePosition {
            x: (2.0 * std::f64::consts::PI * i as f64 / n as f64).cos(),
            y: (2.0 * std::f64::consts::PI * i as f64 / n as f64).sin(),
            z: None,
        })
        .collect();
    BuiltModel {
        hamiltonian: Hamiltonian::new(n, terms),
        layout: LatticeLayout {
            true_positions,
            expected_dim: n.saturating_sub(1),
        },
        label: format!("Random non-local (n={n})"),
    }
}
