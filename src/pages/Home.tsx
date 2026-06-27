import { ExternalLink, Atom, Layers, GitBranch, AlertCircle } from 'lucide-react'
import { Link } from 'react-router-dom'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Separator } from '@/components/ui/separator'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { SpectrumViz } from '@/components/visualizations/SpectrumViz'
import { EntanglementGraphViz } from '@/components/visualizations/EntanglementGraphViz'
import { DecoherenceViz } from '@/components/visualizations/DecoherenceViz'

const sections = [
  {
    id: 'ontology',
    icon: Atom,
    title: 'Minimal Ontology',
    content: (
      <>
        <p>
          Mad-Dog Everettianism takes quantum mechanics at face value: the
          universe is described by a vector <code className="font-mono text-sm bg-muted px-1.5 py-0.5 rounded">|ψ⟩</code> in
          a Hilbert space <code className="font-mono text-sm bg-muted px-1.5 py-0.5 rounded">ℋ</code>, evolving
          under a Hamiltonian <code className="font-mono text-sm bg-muted px-1.5 py-0.5 rounded">Ĥ</code> via
          the Schrödinger equation.
        </p>
        <p className="mt-3">
          Unlike typical presentations, no preferred algebra of observables is
          assumed — no privileged classical variables like position or
          momentum. Space, fields, and particles are all expected to{' '}
          <em>emerge</em> from this austere quantum description.
        </p>
      </>
    ),
  },
  {
    id: 'spectrum',
    icon: Layers,
    title: 'Spectrum & State Components',
    content: (
      <>
        <p>
          In the energy eigenbasis, the Hamiltonian is fixed by its spectrum{' '}
          <code className="font-mono text-sm bg-muted px-1.5 py-0.5 rounded">{'{E₀, E₁, E₂, …}'}</code> and
          the state by its components{' '}
          <code className="font-mono text-sm bg-muted px-1.5 py-0.5 rounded">{'{ψ₀, ψ₁, ψ₂, …}'}</code>.
          These two lists of numbers are, in principle, all the fundamental data.
        </p>
        <p className="mt-3">
          The challenge: how do we recover the rich classical world — with its
          spacetime geometry, localized particles, and definite measurement
          outcomes — from such minimal ingredients?
        </p>
      </>
    ),
  },
  {
    id: 'emergence',
    icon: GitBranch,
    title: 'Emergent Structure',
    content: (
      <>
        <p>
          The Hilbert space is assumed to be <strong>locally
          finite-dimensional</strong>, decomposable into micro-factors{' '}
          <code className="font-mono text-sm bg-muted px-1.5 py-0.5 rounded">ℋ = ⊗ₐ ℋₐ</code>.
          For a suitable Hamiltonian, a natural local decomposition exists where
          physics looks local — a graph of interacting factors.
        </p>
        <p className="mt-3">
          Mutual information between factors weights the graph edges. High
          entanglement implies short emergent distance, yielding spatial
          geometry via multidimensional scaling. Entanglement equilibrium then
          connects geometry to energy-momentum, recovering Einstein&apos;s equation
          at the linearized level.
        </p>
      </>
    ),
  },
  {
    id: 'classicality',
    icon: GitBranch,
    title: 'Emergent Classicality',
    content: (
      <>
        <p>
          Classical behavior arises when systems decohere through environmental
          entanglement. An initially unentangled state evolves to correlate
          object, apparatus, and environment — producing orthogonal branches
          that each look classical.
        </p>
        <p className="mt-3">
          The Born rule <code className="font-mono text-sm bg-muted px-1.5 py-0.5 rounded">p(i) = |ψᵢ|²</code> is
          not postulated but derived from self-locating uncertainty. The
          system/environment split itself emerges from the Hamiltonian via the
          factorization that minimizes entanglement growth.
        </p>
      </>
    ),
  },
  {
    id: 'puzzles',
    icon: AlertCircle,
    title: 'Open Puzzles',
    content: (
      <ul className="space-y-2 list-disc list-inside text-muted-foreground">
        <li>
          <strong className="text-foreground">Lorentz invariance:</strong> No
          unitary representations of the Lorentz group exist on finite-dimensional
          factors. How approximate is Lorentz symmetry?
        </li>
        <li>
          <strong className="text-foreground">Effective field theory:</strong>{' '}
          Can infrared matter degrees of freedom be identified with a quantum
          error-correcting code subspace?
        </li>
        <li>
          <strong className="text-foreground">The problem of time:</strong>{' '}
          In the Wheeler-DeWitt case, what determines the clock subsystem? The
          thermal time hypothesis offers one route.
        </li>
        <li>
          <strong className="text-foreground">Factor count:</strong> The theory
          assumes a fixed tensor-product factorization ℋ = ⊗ₐ ℋₐ — the
          &ldquo;qubits&rdquo; are already there. It does not explain why there
          are this many fundamental degrees of freedom, or whether the universe
          can create new ones over time.
        </li>
      </ul>
    ),
  },
]

function Home() {
  return (
    <div className="flex-1">
      {/* Hero */}
      <header className="border-b">
        <div className="mx-auto max-w-4xl px-6 py-16 text-center">
          <Badge variant="secondary" className="mb-4">
            arXiv:1801.08132
          </Badge>
          <h1 className="text-4xl md:text-5xl font-semibold tracking-tight mb-4">
            Mad-Dog Everettianism
          </h1>
          <p className="text-lg text-muted-foreground max-w-2xl mx-auto mb-2">
            Quantum Mechanics at Its Most Minimal
          </p>
          <p className="text-sm text-muted-foreground mb-6">
            Sean M. Carroll &amp; Ashmeet Singh · Caltech · 2018
          </p>
          <p className="text-base max-w-xl mx-auto mb-8 leading-relaxed">
            From a Hamiltonian spectrum and a state vector to spacetime,
            classicality, and gravity — everything familiar emerges from
            Hilbert space alone.
          </p>
          <div className="flex gap-3 justify-center flex-wrap">
            <Button asChild>
              <a
                href="https://arxiv.org/pdf/1801.08132"
                target="_blank"
                rel="noopener noreferrer"
              >
                Read the Paper
                <ExternalLink className="size-4" />
              </a>
            </Button>
            <Button variant="outline" asChild>
              <Link to="/experiments">Run Experiments</Link>
            </Button>
            <Button variant="outline" asChild>
              <Link to="/lab">Universe Lab (3+1)</Link>
            </Button>
            <Button variant="outline" asChild>
              <a
                href="https://arxiv.org/abs/1801.08132"
                target="_blank"
                rel="noopener noreferrer"
              >
                arXiv Abstract
              </a>
            </Button>
          </div>
        </div>
      </header>

      <main className="mx-auto max-w-4xl px-6 py-12 space-y-16">
        {/* Core idea */}
        <section>
          <h2 className="text-2xl font-semibold mb-4">The Core Idea</h2>
          <Card>
            <CardContent className="pt-6 text-muted-foreground leading-relaxed">
              <p>
                Most quantum theories start with classical variables — positions,
                fields, spacetime — and quantize them. Mad-Dog Everettianism
                inverts this: Nature is quantum from the start. The fundamental
                description is a vector moving smoothly through a very large
                Hilbert space, governed by a Hamiltonian whose spectrum and the
                state&apos;s components in the energy basis constitute the only
                primitive data.
              </p>
              <p className="mt-4">
                The name is inspired by philosopher Owen Flanagan&apos;s
                description of Alex Rosenberg&apos;s philosophy as &ldquo;Mad-Dog
                Naturalism&rdquo; — here pushed to the extreme of quantum
                minimalism.
              </p>
            </CardContent>
          </Card>
        </section>

        {/* Explanatory sections */}
        <section className="space-y-6">
          <h2 className="text-2xl font-semibold">Key Concepts</h2>
          {sections.map((section) => (
            <Card key={section.id}>
              <CardHeader>
                <div className="flex items-center gap-3">
                  <section.icon className="size-5 text-primary" />
                  <CardTitle>{section.title}</CardTitle>
                </div>
              </CardHeader>
              <CardContent className="text-muted-foreground leading-relaxed">
                {section.content}
              </CardContent>
            </Card>
          ))}
        </section>

        <Separator />

        {/* Visualizations */}
        <section className="space-y-6">
          <div>
            <h2 className="text-2xl font-semibold mb-2">Visualizations</h2>
            <p className="text-muted-foreground">
              Illustrative diagrams for the paper&apos;s central constructions.
              Live quantum checks are on the{' '}
              <Link to="/experiments" className="underline hover:text-foreground">
                Experiments
              </Link>{' '}
              tab.
            </p>
          </div>

          <Tabs defaultValue="spectrum">
            <TabsList>
              <TabsTrigger value="spectrum">Minimal Data</TabsTrigger>
              <TabsTrigger value="geometry">Emergent Geometry</TabsTrigger>
              <TabsTrigger value="decoherence">Decoherence</TabsTrigger>
            </TabsList>
            <TabsContent value="spectrum" className="mt-4">
              <SpectrumViz />
            </TabsContent>
            <TabsContent value="geometry" className="mt-4">
              <EntanglementGraphViz />
            </TabsContent>
            <TabsContent value="decoherence" className="mt-4">
              <DecoherenceViz />
            </TabsContent>
          </Tabs>
        </section>

        {/* Roadmap */}
        <section>
          <Card className="border-dashed">
            <CardHeader>
              <CardTitle>What&apos;s Next</CardTitle>
              <CardDescription>
                Planned extensions for this project
              </CardDescription>
            </CardHeader>
            <CardContent>
              <ul className="space-y-2 text-sm text-muted-foreground">
                <li>
                  Scattering phase shift and time delay when two light cones
                  overlap (chain, h sweep)
                </li>
                <li>
                  Two-defect scattering on 2D grids and 3D cubes (currently
                  chain-only)
                </li>
                <li>
                  Effective particle mass from lattice excitation dispersion
                </li>
                <li>
                  Compare factorizations of the same state at different factor
                  counts (embedding / truncation)
                </li>
                <li>
                  3D and torus factorization search targets; QECC probes on
                  non-TFIM models
                </li>
                <li>
                  Explore the 3+1 dashboard at{' '}
                  <Link to="/lab" className="underline hover:text-foreground">
                    /lab
                  </Link>{' '}
                  — signatures S1–S10 and Phase 10 are shipped
                </li>
              </ul>
            </CardContent>
          </Card>
        </section>
      </main>

      <footer className="border-t py-8 text-center text-sm text-muted-foreground">
        <p>
          Based on{' '}
          <a
            href="https://arxiv.org/abs/1801.08132"
            className="underline hover:text-foreground transition-colors"
            target="_blank"
            rel="noopener noreferrer"
          >
            Carroll &amp; Singh (2018)
          </a>
          . Built with React, Vite, and ShadCN.
        </p>
      </footer>
    </div>
  )
}

export default Home
