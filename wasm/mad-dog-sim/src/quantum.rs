//! Minimal quantum state-vector engine — port of `src/sim/quantum.ts`.

use crate::rng::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauliLetter {
    X,
    Y,
    Z,
}

#[derive(Clone, Debug)]
pub struct PauliOp {
    pub qubit: usize,
    pub letter: PauliLetter,
}

#[derive(Clone, Debug)]
pub struct PauliTerm {
    pub coeff: f64,
    pub ops: Vec<PauliOp>,
}

#[derive(Clone, Debug)]
pub struct QuantumState {
    pub n: usize,
    pub dim: usize,
    /// Interleaved [re, im] amplitudes.
    pub data: Vec<f64>,
}

impl QuantumState {
    pub fn zero(n: usize) -> Self {
        let dim = 1usize << n;
        Self {
            n,
            dim,
            data: vec![0.0; 2 * dim],
        }
    }

    pub fn clone_state(&self) -> Self {
        Self {
            n: self.n,
            dim: self.dim,
            data: self.data.clone(),
        }
    }

    pub fn norm(&self) -> f64 {
        let mut sum = 0.0;
        for i in 0..self.dim {
            let re = self.data[2 * i];
            let im = self.data[2 * i + 1];
            sum += re * re + im * im;
        }
        sum.sqrt()
    }

    pub fn normalize(&mut self) {
        let nrm = self.norm();
        if nrm < 1e-300 {
            return;
        }
        let inv = 1.0 / nrm;
        for v in &mut self.data {
            *v *= inv;
        }
    }
}

pub fn make_random_state(n: usize, rng: &mut Rng) -> QuantumState {
    let mut state = QuantumState::zero(n);
    for i in 0..state.dim {
        let u1 = rng.next().max(1e-12);
        let u2 = rng.next();
        let mag = (-2.0 * u1.ln()).sqrt();
        state.data[2 * i] = mag * (2.0 * std::f64::consts::PI * u2).cos();
        state.data[2 * i + 1] = mag * (2.0 * std::f64::consts::PI * u2).sin();
    }
    state.normalize();
    state
}

fn apply_pauli_term(term: &PauliTerm, in_state: &QuantumState, out: &mut [f64]) {
    let data = &in_state.data;
    let dim = in_state.dim;
    let coeff = term.coeff;

    for s in 0..dim {
        let mut target = s;
        let mut pr = 1.0;
        let mut pi = 0.0;

        for op in &term.ops {
            let bit = (s >> op.qubit) & 1;
            match op.letter {
                PauliLetter::X => {
                    target ^= 1 << op.qubit;
                }
                PauliLetter::Y => {
                    target ^= 1 << op.qubit;
                    let fi = if bit == 0 { 1.0 } else { -1.0 };
                    let npr = -pi * fi;
                    let npi = pr * fi;
                    pr = npr;
                    pi = npi;
                }
                PauliLetter::Z => {
                    if bit == 1 {
                        pr = -pr;
                        pi = -pi;
                    }
                }
            }
        }

        let ar = data[2 * s];
        let ai = data[2 * s + 1];
        let cr = coeff * pr;
        let ci = coeff * pi;
        out[2 * target] += cr * ar - ci * ai;
        out[2 * target + 1] += cr * ai + ci * ar;
    }
}

#[derive(Clone, Debug)]
pub struct Hamiltonian {
    pub n: usize,
    pub terms: Vec<PauliTerm>,
}

impl Hamiltonian {
    pub fn new(n: usize, terms: Vec<PauliTerm>) -> Self {
        Self { n, terms }
    }

    pub fn apply(&self, state: &QuantumState) -> QuantumState {
        let mut result = QuantumState::zero(self.n);
        for term in &self.terms {
            apply_pauli_term(term, state, &mut result.data);
        }
        result
    }

    pub fn expectation(&self, state: &QuantumState) -> f64 {
        let h_psi = self.apply(state);
        let mut re = 0.0;
        for i in 0..state.dim {
            let sr = state.data[2 * i];
            let si = state.data[2 * i + 1];
            let hr = h_psi.data[2 * i];
            let hi = h_psi.data[2 * i + 1];
            re += sr * hr + si * hi;
        }
        re
    }

    pub fn estimate_spectral_radius(&self, rng: &mut Rng, iters: usize) -> f64 {
        let mut v = make_random_state(self.n, rng);
        let mut lambda = 0.0;
        for _ in 0..iters {
            let hv = self.apply(&v);
            lambda = hv.norm();
            if lambda < 1e-300 {
                break;
            }
            let mut hv = hv;
            hv.normalize();
            v = hv;
        }
        lambda
    }
}

pub fn ground_state(
    h: &Hamiltonian,
    rng: &mut Rng,
    max_iters: usize,
    tol: f64,
) -> (QuantumState, f64, usize) {
    let radius = h.estimate_spectral_radius(rng, 30).max(1e-6);
    let dt = 0.5 / radius;

    let mut state = make_random_state(h.n, rng);
    let mut prev_energy = f64::INFINITY;
    let mut energy = h.expectation(&state);
    let mut iters = 0usize;

    while iters < max_iters {
        let h_psi = h.apply(&state);
        for i in 0..state.dim {
            state.data[2 * i] -= dt * h_psi.data[2 * i];
            state.data[2 * i + 1] -= dt * h_psi.data[2 * i + 1];
        }
        state.normalize();
        energy = h.expectation(&state);
        if (prev_energy - energy).abs() < tol {
            iters += 1;
            break;
        }
        prev_energy = energy;
        iters += 1;
    }

    (state, energy, iters)
}

pub fn expectation_z(state: &QuantumState, q: usize) -> f64 {
    let mut sum = 0.0;
    for s in 0..state.dim {
        let sign = if (s >> q) & 1 == 1 { -1.0 } else { 1.0 };
        let re = state.data[2 * s];
        let im = state.data[2 * s + 1];
        sum += sign * (re * re + im * im);
    }
    sum
}

pub fn kick_x(state: &QuantumState, qubit: usize) -> QuantumState {
    let mut kicked = state.clone_state();
    for s in 0..state.dim {
        if (s >> qubit) & 1 != 0 {
            continue;
        }
        let t = s | (1 << qubit);
        kicked.data[2 * t] = state.data[2 * s];
        kicked.data[2 * t + 1] = state.data[2 * s + 1];
        kicked.data[2 * s] = 0.0;
        kicked.data[2 * s + 1] = 0.0;
    }
    kicked
}

pub fn evolve_step(h: &Hamiltonian, state: &QuantumState, dt: f64, order: usize) -> QuantumState {
    let mut out = state.clone_state();
    let mut term = state.clone_state();
    for m in 1..=order {
        let h_term = h.apply(&term);
        let f = dt / m as f64;
        let mut next = h_term.clone_state();
        for i in 0..state.dim {
            let a = h_term.data[2 * i];
            let b = h_term.data[2 * i + 1];
            next.data[2 * i] = f * b;
            next.data[2 * i + 1] = -f * a;
        }
        for i in 0..2 * state.dim {
            out.data[i] += next.data[i];
        }
        term = next;
    }
    out
}

pub fn evolve_interval(
    h: &Hamiltonian,
    state: &QuantumState,
    dt: f64,
    radius: f64,
    order: usize,
) -> QuantumState {
    let sub = ((dt.abs() * radius) / 0.2).ceil().max(1.0) as usize;
    let micro = dt / sub as f64;
    let mut psi = state.clone_state();
    for _ in 0..sub {
        psi = evolve_step(h, &psi, micro, order);
    }
    psi
}
