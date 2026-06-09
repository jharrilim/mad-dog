export interface EnergyLevel {
  n: number
  energy: number
  amplitude: { re: number; im: number }
}

export const energyLevels: EnergyLevel[] = [
  { n: 0, energy: 0.0, amplitude: { re: 0.55, im: 0.12 } },
  { n: 1, energy: 1.2, amplitude: { re: 0.38, im: -0.22 } },
  { n: 2, energy: 2.8, amplitude: { re: 0.15, im: 0.31 } },
  { n: 3, energy: 5.1, amplitude: { re: -0.08, im: 0.18 } },
  { n: 4, energy: 8.4, amplitude: { re: 0.04, im: -0.06 } },
  { n: 5, energy: 12.7, amplitude: { re: 0.02, im: 0.03 } },
]

export interface GraphNode {
  id: string
  label: string
  x: number
  y: number
}

export interface GraphEdge {
  from: string
  to: string
  mutualInfo: number
}

export const graphNodes: GraphNode[] = [
  { id: 'a', label: 'α', x: 200, y: 80 },
  { id: 'b', label: 'β', x: 320, y: 140 },
  { id: 'c', label: 'γ', x: 140, y: 200 },
  { id: 'd', label: 'δ', x: 280, y: 240 },
  { id: 'e', label: 'ε', x: 400, y: 200 },
  { id: 'f', label: 'ζ', x: 200, y: 300 },
]

export const graphEdges: GraphEdge[] = [
  { from: 'a', to: 'b', mutualInfo: 0.85 },
  { from: 'a', to: 'c', mutualInfo: 0.72 },
  { from: 'b', to: 'd', mutualInfo: 0.68 },
  { from: 'c', to: 'd', mutualInfo: 0.55 },
  { from: 'd', to: 'e', mutualInfo: 0.45 },
  { from: 'c', to: 'f', mutualInfo: 0.38 },
  { from: 'd', to: 'f', mutualInfo: 0.62 },
  { from: 'b', to: 'e', mutualInfo: 0.25 },
  { from: 'a', to: 'e', mutualInfo: 0.12 },
]

export interface DecoherenceStep {
  step: number
  label: string
  equation: string
  description: string
  branches?: { label: string; amplitude: string }[]
}

export const decoherenceSteps: DecoherenceStep[] = [
  {
    step: 0,
    label: 'Initial',
    equation: '|ψ⟩ = (α|+⟩_q + β|−⟩_q) ⊗ |0⟩_a ⊗ |0⟩_e',
    description:
      'Object, apparatus, and environment start unentangled. The quantum object is in a superposition.',
  },
  {
    step: 1,
    label: 'Measurement',
    equation:
      '→ (α|+⟩_q|+⟩_a + β|−⟩_q|−⟩_a) ⊗ |0⟩_e',
    description:
      'The apparatus becomes correlated with the object. Superposition is now shared between object and apparatus.',
  },
  {
    step: 2,
    label: 'Decoherence',
    equation:
      '→ α|+⟩_q|+⟩_a|+⟩_e + β|−⟩_q|−⟩_a|−⟩_e',
    description:
      'The environment entangles with the apparatus, splitting the wave function into orthogonal branches. Each branch appears classical.',
    branches: [
      { label: 'Branch +', amplitude: '|α|²' },
      { label: 'Branch −', amplitude: '|β|²' },
    ],
  },
]
