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

/// Reusable scratch for in-place Hamiltonian application and Taylor evolution.
pub struct EvolveScratch {
    pub h_psi: Vec<f64>,
    pub term: Vec<f64>,
}

impl EvolveScratch {
    pub fn new(dim: usize) -> Self {
        Self {
            h_psi: vec![0.0; 2 * dim],
            term: vec![0.0; 2 * dim],
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

fn apply_tfim_term_into(
    term: &PauliTerm,
    data: &[f64],
    dim: usize,
    out: &mut [f64],
) {
    let coeff = term.coeff;
    if term.ops.len() == 1 {
        let q = term.ops[0].qubit;
        let mask = 1usize << q;
        for s in 0..dim {
            let t = s ^ mask;
            let ar = data[2 * s];
            let ai = data[2 * s + 1];
            out[2 * t] += coeff * ar;
            out[2 * t + 1] += coeff * ai;
        }
    } else if term.ops.len() == 2
        && term.ops[0].letter == PauliLetter::Z
        && term.ops[1].letter == PauliLetter::Z
    {
        let q1 = term.ops[0].qubit;
        let q2 = term.ops[1].qubit;
        for s in 0..dim {
            let b1 = (s >> q1) & 1;
            let b2 = (s >> q2) & 1;
            let phase = if b1 == b2 { 1.0 } else { -1.0 };
            let ar = data[2 * s];
            let ai = data[2 * s + 1];
            out[2 * s] += coeff * phase * ar;
            out[2 * s + 1] += coeff * phase * ai;
        }
    } else {
        let state = QuantumState {
            n: (dim as f64).log2() as usize,
            dim,
            data: data.to_vec(),
        };
        apply_pauli_term(term, &state, out);
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

    fn apply_into_raw(&self, data: &[f64], dim: usize, n: usize, out: &mut [f64]) {
        out.fill(0.0);
        if self.is_tfim_like() {
            for term in &self.terms {
                apply_tfim_term_into(term, data, dim, out);
            }
        } else {
            let state = QuantumState {
                n,
                dim,
                data: data.to_vec(),
            };
            for term in &self.terms {
                apply_pauli_term(term, &state, out);
            }
        }
    }

    fn is_tfim_like(&self) -> bool {
        self.terms.iter().all(|term| {
            (term.ops.len() == 1 && term.ops[0].letter == PauliLetter::X)
                || (term.ops.len() == 2
                    && term.ops[0].letter == PauliLetter::Z
                    && term.ops[1].letter == PauliLetter::Z)
        })
    }

    pub fn apply_into(&self, state: &QuantumState, out: &mut [f64]) {
        self.apply_into_raw(&state.data, state.dim, state.n, out);
    }

    pub fn expectation_with_scratch(&self, state: &QuantumState, scratch: &mut [f64]) -> f64 {
        self.apply_into(state, scratch);
        let mut re = 0.0;
        for i in 0..state.dim {
            let sr = state.data[2 * i];
            let si = state.data[2 * i + 1];
            let hr = scratch[2 * i];
            let hi = scratch[2 * i + 1];
            re += sr * hr + si * hi;
        }
        re
    }

    pub fn expectation(&self, state: &QuantumState) -> f64 {
        let mut scratch = vec![0.0; 2 * state.dim];
        self.expectation_with_scratch(state, &mut scratch)
    }

    pub fn spectral_radius(&self, rng: &mut Rng) -> f64 {
        self.estimate_spectral_radius(rng, 30).max(1e-6)
    }

    pub fn estimate_spectral_radius(&self, rng: &mut Rng, iters: usize) -> f64 {
        let mut v = make_random_state(self.n, rng);
        let mut scratch = EvolveScratch::new(v.dim);
        let mut lambda = 0.0;
        for _ in 0..iters {
            self.apply_into(&v, &mut scratch.h_psi);
            lambda = scratch.h_psi.chunks(2).fold(0.0, |acc, c| {
                acc + c[0] * c[0] + c[1] * c[1]
            }).sqrt();
            if lambda < 1e-300 {
                break;
            }
            let inv = 1.0 / lambda;
            for (i, chunk) in scratch.h_psi.chunks(2).enumerate() {
                v.data[2 * i] = chunk[0] * inv;
                v.data[2 * i + 1] = chunk[1] * inv;
            }
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
    let radius = h.spectral_radius(rng);
    let dt = 0.5 / radius;

    let mut state = make_random_state(h.n, rng);
    let mut scratch = EvolveScratch::new(state.dim);
    let mut prev_energy = f64::INFINITY;
    let mut energy = h.expectation_with_scratch(&state, &mut scratch.h_psi);
    let mut iters = 0usize;

    while iters < max_iters {
        h.apply_into(&state, &mut scratch.h_psi);
        for i in 0..state.dim {
            state.data[2 * i] -= dt * scratch.h_psi[2 * i];
            state.data[2 * i + 1] -= dt * scratch.h_psi[2 * i + 1];
        }
        state.normalize();
        energy = h.expectation_with_scratch(&state, &mut scratch.h_psi);
        if (prev_energy - energy).abs() < tol {
            iters += 1;
            break;
        }
        prev_energy = energy;
        iters += 1;
    }

    (state, energy, iters)
}

fn inner_product(a: &QuantumState, b: &QuantumState) -> f64 {
    let mut re = 0.0;
    for i in 0..a.dim {
        let ar = a.data[2 * i];
        let ai = a.data[2 * i + 1];
        let br = b.data[2 * i];
        let bi = b.data[2 * i + 1];
        re += ar * br + ai * bi;
    }
    re
}

fn project_orthogonal(state: &mut QuantumState, basis: &[QuantumState]) {
    for b in basis {
        let inner = inner_product(state, b);
        for i in 0..state.dim {
            state.data[2 * i] -= inner * b.data[2 * i];
            state.data[2 * i + 1] -= inner * b.data[2 * i + 1];
        }
    }
    state.normalize();
}

/// Low-energy eigenstates via imaginary-time descent with Gram–Schmidt deflation.
pub fn low_energy_states(h: &Hamiltonian, k: usize, seed: u32) -> Vec<(QuantumState, f64)> {
    let dim = 1usize << h.n;
    let k = k.max(1).min(dim);
    let mut rng = Rng::new(seed);
    let radius = h.spectral_radius(&mut rng);
    let dt = 0.5 / radius;
    let mut found: Vec<(QuantumState, f64)> = Vec::new();
    let mut scratch = EvolveScratch::new(dim);

    for _ in 0..k {
        let mut state = make_random_state(h.n, &mut rng);
        let prior: Vec<QuantumState> = found.iter().map(|(s, _)| s.clone_state()).collect();
        project_orthogonal(&mut state, &prior);

        let mut prev_energy = f64::INFINITY;
        for _ in 0..4000 {
            h.apply_into(&state, &mut scratch.h_psi);
            for i in 0..state.dim {
                state.data[2 * i] -= dt * scratch.h_psi[2 * i];
                state.data[2 * i + 1] -= dt * scratch.h_psi[2 * i + 1];
            }
            project_orthogonal(&mut state, &prior);
            let energy = h.expectation_with_scratch(&state, &mut scratch.h_psi);
            if (prev_energy - energy).abs() < 1e-8 {
                break;
            }
            prev_energy = energy;
        }
        let energy = h.expectation_with_scratch(&state, &mut scratch.h_psi);
        found.push((state, energy));
    }
    found
}

/// Dense matrix representation (re, im) in computational basis.
pub fn hamiltonian_dense(h: &Hamiltonian) -> (Vec<Vec<f64>>, Vec<Vec<f64>>) {
    let dim = 1usize << h.n;
    let mut re = vec![vec![0.0; dim]; dim];
    let mut im = vec![vec![0.0; dim]; dim];
    let mut scratch = vec![0.0; 2 * dim];
    for col in 0..dim {
        let mut basis = QuantumState::zero(h.n);
        basis.data[2 * col] = 1.0;
        h.apply_into(&basis, &mut scratch);
        for row in 0..dim {
            re[row][col] = scratch[2 * row];
            im[row][col] = scratch[2 * row + 1];
        }
    }
    (re, im)
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

pub fn expectation_x(state: &QuantumState, q: usize) -> f64 {
    let mask = 1 << q;
    let mut sum = 0.0;
    for s in 0..state.dim {
        if (s & mask) != 0 {
            continue;
        }
        let t = s | mask;
        let ar = state.data[2 * s];
        let ai = state.data[2 * s + 1];
        let br = state.data[2 * t];
        let bi = state.data[2 * t + 1];
        sum += 2.0 * (ar * br + ai * bi);
    }
    sum
}

/// All ⟨Z_q⟩ in one pass over the state vector.
pub fn expectation_z_all(state: &QuantumState) -> Vec<f64> {
    let n = state.n;
    let mut out = vec![0.0; n];
    for s in 0..state.dim {
        let re = state.data[2 * s];
        let im = state.data[2 * s + 1];
        let prob = re * re + im * im;
        for q in 0..n {
            if (s >> q) & 1 == 1 {
                out[q] -= prob;
            } else {
                out[q] += prob;
            }
        }
    }
    out
}

pub fn signal_from_z(z: &[f64], ref_z: &[f64]) -> Vec<f64> {
    z.iter()
        .zip(ref_z.iter())
        .map(|(a, b)| (a - b).abs())
        .collect()
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

fn multiply_minus_i(data: &[f64], factor: f64, out: &mut [f64]) {
    for i in 0..(data.len() / 2) {
        let a = data[2 * i];
        let b = data[2 * i + 1];
        out[2 * i] = factor * b;
        out[2 * i + 1] = -factor * a;
    }
}

pub fn evolve_step_inplace(
    h: &Hamiltonian,
    psi: &mut QuantumState,
    dt: f64,
    order: usize,
    scratch: &mut EvolveScratch,
) {
    scratch.term.copy_from_slice(&psi.data);
    for m in 1..=order {
        h.apply_into_raw(&scratch.term, psi.dim, psi.n, &mut scratch.h_psi);
        multiply_minus_i(&scratch.h_psi, dt / m as f64, &mut scratch.term);
        for i in 0..psi.data.len() {
            psi.data[i] += scratch.term[i];
        }
    }
}

pub fn evolve_interval_inplace(
    h: &Hamiltonian,
    psi: &mut QuantumState,
    dt: f64,
    radius: f64,
    order: usize,
    scratch: &mut EvolveScratch,
) {
    let sub = ((dt.abs() * radius) / 0.2).ceil().max(1.0) as usize;
    let micro = dt / sub as f64;
    for _ in 0..sub {
        evolve_step_inplace(h, psi, micro, order, scratch);
    }
}

pub fn evolve_interval(
    h: &Hamiltonian,
    state: &QuantumState,
    dt: f64,
    radius: f64,
    order: usize,
) -> QuantumState {
    let mut psi = state.clone_state();
    let mut scratch = EvolveScratch::new(state.dim);
    evolve_interval_inplace(h, &mut psi, dt, radius, order, &mut scratch);
    psi
}
