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

/// 3D transverse-field Ising cube (open boundary).
pub fn tfim_cube(lx: usize, ly: usize, lz: usize, j: f64, h: f64) -> BuiltModel {
    let n = lx * ly * lz;
    let idx = |x: usize, y: usize, z: usize| z * (lx * ly) + y * lx + x;
    let mut terms = Vec::new();

    for z in 0..lz {
        for y in 0..ly {
            for x in 0..lx {
                let q = idx(x, y, z);
                if x + 1 < lx {
                    terms.push(PauliTerm {
                        coeff: -j,
                        ops: vec![
                            PauliOp {
                                qubit: q,
                                letter: PauliLetter::Z,
                            },
                            PauliOp {
                                qubit: idx(x + 1, y, z),
                                letter: PauliLetter::Z,
                            },
                        ],
                    });
                }
                if y + 1 < ly {
                    terms.push(PauliTerm {
                        coeff: -j,
                        ops: vec![
                            PauliOp {
                                qubit: q,
                                letter: PauliLetter::Z,
                            },
                            PauliOp {
                                qubit: idx(x, y + 1, z),
                                letter: PauliLetter::Z,
                            },
                        ],
                    });
                }
                if z + 1 < lz {
                    terms.push(PauliTerm {
                        coeff: -j,
                        ops: vec![
                            PauliOp {
                                qubit: q,
                                letter: PauliLetter::Z,
                            },
                            PauliOp {
                                qubit: idx(x, y, z + 1),
                                letter: PauliLetter::Z,
                            },
                        ],
                    });
                }
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
    for z in 0..lz {
        for y in 0..ly {
            for x in 0..lx {
                true_positions.push(TruePosition {
                    x: x as f64,
                    y: y as f64,
                    z: Some(z as f64),
                });
            }
        }
    }

    BuiltModel {
        hamiltonian: Hamiltonian::new(n, terms),
        layout: LatticeLayout {
            true_positions,
            expected_dim: 3,
        },
        label: format!("TFIM cube ({lx}x{ly}x{lz})"),
    }
}

/// Cube lattice neighbour pairs for drawing edges.
pub fn cube_edges(lx: usize, ly: usize, lz: usize) -> Vec<[usize; 2]> {
    let idx = |x: usize, y: usize, z: usize| z * (lx * ly) + y * lx + x;
    let mut edges = Vec::new();
    for z in 0..lz {
        for y in 0..ly {
            for x in 0..lx {
                let q = idx(x, y, z);
                if x + 1 < lx {
                    edges.push([q, idx(x + 1, y, z)]);
                }
                if y + 1 < ly {
                    edges.push([q, idx(x, y + 1, z)]);
                }
                if z + 1 < lz {
                    edges.push([q, idx(x, y, z + 1)]);
                }
            }
        }
    }
    edges
}
