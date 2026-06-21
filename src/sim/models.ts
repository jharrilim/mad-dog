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
  truePositions: { x: number; y: number; z?: number }[]
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
 * 3D transverse-field Ising cube (open boundary). Qubits indexed
 * idx(x,y,z) = z*(lx*ly) + y*lx + x. Couplings along x, y, z nearest
 * neighbours -> emergent 3D geometry.
 */
export function tfimCube(lx: number, ly: number, lz: number, j = 1, h = 0.5): BuiltModel {
  const n = lx * ly * lz
  const idx = (x: number, y: number, z: number) => z * (lx * ly) + y * lx + x
  const terms: PauliTerm[] = []

  for (let z = 0; z < lz; z++) {
    for (let y = 0; y < ly; y++) {
      for (let x = 0; x < lx; x++) {
        const q = idx(x, y, z)
        if (x + 1 < lx) {
          terms.push({
            coeff: -j,
            ops: [
              { qubit: q, letter: 'Z' },
              { qubit: idx(x + 1, y, z), letter: 'Z' },
            ],
          })
        }
        if (y + 1 < ly) {
          terms.push({
            coeff: -j,
            ops: [
              { qubit: q, letter: 'Z' },
              { qubit: idx(x, y + 1, z), letter: 'Z' },
            ],
          })
        }
        if (z + 1 < lz) {
          terms.push({
            coeff: -j,
            ops: [
              { qubit: q, letter: 'Z' },
              { qubit: idx(x, y, z + 1), letter: 'Z' },
            ],
          })
        }
      }
    }
  }
  for (let i = 0; i < n; i++) {
    terms.push({ coeff: -h, ops: [{ qubit: i, letter: 'X' }] })
  }

  const truePositions: { x: number; y: number; z: number }[] = []
  for (let z = 0; z < lz; z++) {
    for (let y = 0; y < ly; y++) {
      for (let x = 0; x < lx; x++) truePositions.push({ x, y, z })
    }
  }

  return {
    hamiltonian: new Hamiltonian(n, terms),
    layout: { truePositions, expectedDim: 3 },
    label: `TFIM cube (${lx}x${ly}x${lz})`,
  }
}

/** Cube lattice neighbour pairs for drawing edges. */
export function cubeEdges(lx: number, ly: number, lz: number): [number, number][] {
  const idx = (x: number, y: number, z: number) => z * (lx * ly) + y * lx + x
  const edges: [number, number][] = []
  for (let z = 0; z < lz; z++) {
    for (let y = 0; y < ly; y++) {
      for (let x = 0; x < lx; x++) {
        const q = idx(x, y, z)
        if (x + 1 < lx) edges.push([q, idx(x + 1, y, z)])
        if (y + 1 < ly) edges.push([q, idx(x, y + 1, z)])
        if (z + 1 < lz) edges.push([q, idx(x, y, z + 1)])
      }
    }
  }
  return edges
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
