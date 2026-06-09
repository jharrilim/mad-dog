/**
 * Hamiltonian model builders.
 *
 * The point of the simulator is to contrast Hamiltonians that are LOCAL with
 * respect to a chosen tensor factorization (so a low-dimensional geometry
 * should emerge from entanglement) against GENERIC/non-local ones (where no
 * clean low-dimensional space is expected to emerge).
 */

import { Hamiltonian, type PauliTerm } from './quantum.ts'

export interface LatticeLayout {
  /** "True" coordinates of each qubit, used only to score the recovered geometry. */
  truePositions: { x: number; y: number }[]
  /** Human label for the expected emergent dimension. */
  expectedDim: number
}

export interface BuiltModel {
  hamiltonian: Hamiltonian
  layout: LatticeLayout
  label: string
}

/**
 * 1D transverse-field Ising chain (open boundary):
 *   H = -J sum_<i,i+1> Z_i Z_{i+1} - h sum_i X_i
 * Ground state obeys an area law -> nearest-neighbour mutual information
 * dominates -> emergent 1D line.
 */
export function tfimChain(n: number, j = 1, h = 0.5): BuiltModel {
  const terms: PauliTerm[] = []
  for (let i = 0; i < n - 1; i++) {
    terms.push({
      coeff: -j,
      ops: [
        { qubit: i, letter: 'Z' },
        { qubit: i + 1, letter: 'Z' },
      ],
    })
  }
  for (let i = 0; i < n; i++) {
    terms.push({ coeff: -h, ops: [{ qubit: i, letter: 'X' }] })
  }
  const truePositions = Array.from({ length: n }, (_, i) => ({ x: i, y: 0 }))
  return {
    hamiltonian: new Hamiltonian(n, terms),
    layout: { truePositions, expectedDim: 1 },
    label: `TFIM chain (n=${n})`,
  }
}

/**
 * 2D transverse-field Ising grid (open boundary). Qubits are indexed
 * row-major. Couplings act on horizontal and vertical nearest neighbours, so
 * the emergent geometry should be 2D.
 */
export function tfimGrid(rows: number, cols: number, j = 1, h = 0.5): BuiltModel {
  const n = rows * cols
  const idx = (r: number, c: number) => r * cols + c
  const terms: PauliTerm[] = []
  for (let r = 0; r < rows; r++) {
    for (let c = 0; c < cols; c++) {
      if (c + 1 < cols) {
        terms.push({
          coeff: -j,
          ops: [
            { qubit: idx(r, c), letter: 'Z' },
            { qubit: idx(r, c + 1), letter: 'Z' },
          ],
        })
      }
      if (r + 1 < rows) {
        terms.push({
          coeff: -j,
          ops: [
            { qubit: idx(r, c), letter: 'Z' },
            { qubit: idx(r + 1, c), letter: 'Z' },
          ],
        })
      }
    }
  }
  for (let i = 0; i < n; i++) {
    terms.push({ coeff: -h, ops: [{ qubit: i, letter: 'X' }] })
  }
  const truePositions: { x: number; y: number }[] = []
  for (let r = 0; r < rows; r++) {
    for (let c = 0; c < cols; c++) truePositions.push({ x: c, y: r })
  }
  return {
    hamiltonian: new Hamiltonian(n, terms),
    layout: { truePositions, expectedDim: 2 },
    label: `TFIM grid (${rows}x${cols})`,
  }
}

/**
 * Generic non-local Hamiltonian: random two-body couplings between ALL pairs of
 * qubits plus random fields. There is no factorization in which this looks
 * local, so we do not expect a clean low-dimensional emergent geometry.
 */
export function randomNonlocal(n: number, rng: () => number): BuiltModel {
  const terms: PauliTerm[] = []
  const letters: ('X' | 'Y' | 'Z')[] = ['X', 'Y', 'Z']
  for (let i = 0; i < n; i++) {
    for (let k = i + 1; k < n; k++) {
      const pa = letters[Math.floor(rng() * 3)]
      const pb = letters[Math.floor(rng() * 3)]
      terms.push({
        coeff: rng() * 2 - 1,
        ops: [
          { qubit: i, letter: pa },
          { qubit: k, letter: pb },
        ],
      })
    }
    terms.push({
      coeff: rng() * 2 - 1,
      ops: [{ qubit: i, letter: letters[Math.floor(rng() * 3)] }],
    })
  }
  // No meaningful "true" geometry; place qubits arbitrarily for plotting.
  const truePositions = Array.from({ length: n }, (_, i) => ({
    x: Math.cos((2 * Math.PI * i) / n),
    y: Math.sin((2 * Math.PI * i) / n),
  }))
  return {
    hamiltonian: new Hamiltonian(n, terms),
    layout: { truePositions, expectedDim: n - 1 },
    label: `Random non-local (n=${n})`,
  }
}
