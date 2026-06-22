import { Link } from 'react-router-dom'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'

function M({ children }: { children: React.ReactNode }) {
  return (
    <code className="font-mono text-sm bg-muted px-1.5 py-0.5 rounded">
      {children}
    </code>
  )
}

type Entry = { term: string; body: React.ReactNode }

type Section = { title: string; description?: string; entries: Entry[] }

const sections: Section[] = [
  {
    title: 'Fundamentals',
    description:
      'The Mad-Dog starting point — what the simulator treats as primitive.',
    entries: [
      {
        term: 'Mad-Dog Everettianism',
        body: (
          <>
            Carroll &amp; Singh&apos;s program: quantum mechanics with no built-in
            classical variables. The universe is a vector{' '}
            <M>|ψ⟩</M> in a Hilbert space <M>ℋ</M>, evolving under a Hamiltonian{' '}
            <M>Ĥ</M>. Space, time, particles, and measurement outcomes are
            expected to emerge from entanglement and dynamics on a suitable
            factorization.
          </>
        ),
      },
      {
        term: 'Hilbert space (ℋ)',
        body: (
          <>
            The state space. In this repo it is finite-dimensional:{' '}
            <M>dim ℋ = 2ⁿ</M> for <M>n</M> qubits. The state is a complex
            amplitude vector stored as <M>Float64Array</M> in Rust/WASM.
          </>
        ),
      },
      {
        term: 'Factor / site / qubit',
        body: (
          <>
            A tensor factor <M>ℋₐ</M> in the decomposition{' '}
            <M>ℋ = ⊗ₐ ℋₐ</M>. Each factor is a two-level system (qubit). In
            code, a &ldquo;site&rdquo; is an index into this factorization — the
            labeling that makes the Hamiltonian look local.
          </>
        ),
      },
      {
        term: 'Hamiltonian (Ĥ)',
        body: (
          <>
            A Hermitian operator generating time evolution. Implemented as a
            Pauli sum: each term is a coefficient times a tensor product of{' '}
            <M>X</M>, <M>Y</M>, <M>Z</M> on selected sites. The spectrum{' '}
            <M>{'{E₀, E₁, …}'}</M> and state components in the energy basis are
            the primitive data in the paper&apos;s picture.
          </>
        ),
      },
      {
        term: 'Ground state',
        body: (
          <>
            A low-energy eigenstate of <M>Ĥ</M>, found here by imaginary-time
            evolution <M>(1 − dt·H)</M> plus normalization. Used for emergent
            geometry demos and holography baselines.
          </>
        ),
      },
    ],
  },
  {
    title: 'Models & parameters',
    description: 'Concrete Hamiltonians and knobs in the UI and runners.',
    entries: [
      {
        term: 'TFIM (transverse-field Ising model)',
        body: (
          <>
            Default model:{' '}
            <M>H = −J Σ_⟨i,j⟩ ZᵢZⱼ − h Σᵢ Xᵢ</M>. Neighbouring spins prefer
            aligned <M>Z</M>; transverse field <M>h</M> mixes spin via{' '}
            <M>X</M>. Variants: open chain, 2D grid, 3D cube (
            <M>tfimChain</M>, <M>tfimGrid</M>, <M>tfimCube</M>).
          </>
        ),
      },
      {
        term: 'J (coupling)',
        body: (
          <>
            Ising strength between neighbours. Default <M>J = 1</M>. In code:
            parameter <M>j</M> on model constructors.
          </>
        ),
      },
      {
        term: 'h / field (transverse field)',
        body: (
          <>
            Strength of the <M>X</M> term. Controls the phase: ordered (
            <M>h ≲ 1</M>) vs paramagnetic (<M>h ≳ 1</M>). UI label{' '}
            <M>field</M>. Critical point for 1D TFIM at <M>h = 1</M> (for{' '}
            <M>J = 1</M>).
          </>
        ),
      },
      {
        term: 'Ordered vs paramagnetic phase',
        body: (
          <>
            <strong>Ordered</strong> (<M>h</M> small): long-range <M>Z</M>{' '}
            correlations; sharp quench worldlines; ground-state MDS can be
            messier. <strong>Paramagnetic</strong> (<M>h</M> large):{' '}
            <M>Z</M> correlations decay with distance; clean ground-state
            emergent geometry (why <M>h = 1.5</M> is common in checks); quench
            tracks muddier.
          </>
        ),
      },
      {
        term: 'Δt (clock step)',
        body: (
          <>
            Discrete time step for Schrödinger evolution{' '}
            <M>{'e^{-iHΔt}'}</M>. Must keep <M>|H|·Δt</M> small enough for the
            Taylor-series integrator; typical values 0.2–0.25.
          </>
        ),
      },
    ],
  },
  {
    title: 'Emergent space',
    description: 'How spatial geometry is reconstructed from entanglement.',
    entries: [
      {
        term: 'Reduced density matrix (RDM)',
        body: (
          <>
            Partial trace of <M>|ψ⟩⟨ψ|</M> to a subset of sites. Single-site
            and two-site RDMs feed entropy and mutual-information calculations.
          </>
        ),
      },
      {
        term: 'Entropy Sₐ',
        body: (
          <>
            Von Neumann entropy of the single-site RDM at factor <M>a</M>:{' '}
            <M>Sₐ = −Tr(ρₐ log ρₐ)</M>.
          </>
        ),
      },
      {
        term: 'Mutual information I(a:b)',
        body: (
          <>
            <M>{'I(a:b) = Sₐ + Sᵦ − S_{ab}'}</M>, clamped ≥ 0. Measures how much
            knowing one factor reduces uncertainty about another. High MI → short
            emergent distance.
          </>
        ),
      },
      {
        term: 'MI distance',
        body: (
          <>
            <M>d(a,b) = −ξ ln(I(a:b) / I_max)</M>; negligible MI is capped at a
            large distance. Gauge-invariant (no MDS needed). Used in light-cone
            compare and geometry-stability tests.
          </>
        ),
      },
      {
        term: 'MDS (multidimensional scaling)',
        body: (
          <>
            Classical embedding: given pairwise distances, find coordinates
            whose Euclidean distances approximate them. Applied to the MI
            distance matrix to produce emergent positions.
          </>
        ),
      },
      {
        term: 'Emergent dimension',
        body: (
          <>
            Estimated from the Gram eigenvalue spectrum after MDS — largest
            relative scree gap, with Kaiser fallback when the first gap is weak.
            Reported as <M>emergentDim</M>. Small systems (e.g. 8 qubits in 3D)
            can be inconclusive.
          </>
        ),
      },
      {
        term: 'Procrustes alignment',
        body: (
          <>
            Rotates/reflects each slice&apos;s MDS coords to match{' '}
            <M>truePositions</M> (known lattice layout). Used only for
            visualization stability across time — not fed to the physics.
          </>
        ),
      },
      {
        term: 'truePositions',
        body: (
          <>
            The model&apos;s built-in lattice coordinates (<M>{'{x, y, z}'}</M>{' '}
            on cubes). Used for Procrustes alignment and drawing neighbour
            edges — not read off by the emergence pipeline.
          </>
        ),
      },
      {
        term: 'Gauge freedom',
        body: (
          <>
            MDS coordinates are defined only up to rotation and reflection.
            Relative MI distances and eigenvalue spectra are gauge-free;
            absolute coords are not.
          </>
        ),
      },
    ],
  },
  {
    title: 'Emergent time & spacetime',
    description: 'Clock readings, slices, and causal structure.',
    entries: [
      {
        term: 'Page–Wootters time',
        body: (
          <>
            No external time parameter. History is built from a timeless global
            state conditioned on an internal clock: slice <M>k</M> at emergent
            time <M>t_k = k·Δt</M>. Implemented by evolving <M>|ψ₀⟩</M> under{' '}
            <M>H</M> for <M>k</M> steps and running the geometry pipeline on
            each slice.
          </>
        ),
      },
      {
        term: 'Clock slice / spacetime slice',
        body: (
          <>
            The system state (and optional MDS embedding) at one clock reading{' '}
            <M>k</M>. A trajectory of slices forms emergent history.
          </>
        ),
      },
      {
        term: 'Light cone',
        body: (
          <>
            The causal front of a localized disturbance: signal arrives later at
            sites farther from the source. Visible on site × time heatmaps. Fitted
            velocity uses lattice distance (chain or Manhattan), not emergent MDS
            coords — see{' '}
            <Link to="/experiments" className="underline hover:text-foreground">
              Light cone compare
            </Link>{' '}
            for MI vs lattice agreement.
          </>
        ),
      },
      {
        term: 'Lieb–Robinson (LR) velocity',
        body: (
          <>
            Finite speed limit for information propagation in local Hamiltonians.
            Measured by <M>measureLightCone</M>: first arrival time of signal above
            threshold at each site, fit <M>distance = v·t</M>.
          </>
        ),
      },
      {
        term: 'Relational time',
        body: (
          <>
            Two (or more) clock readouts on the same trajectory — e.g. uniform
            ticks vs physical ticks when local signal crosses a threshold. Clocks
            at the defect stay synchronized; edge clocks desynchronize when the
            front has not arrived.
          </>
        ),
      },
      {
        term: '3+1 (in this repo)',
        body: (
          <>
            Three emergent spatial dimensions (MDS) plus one emergent time axis on
            the scrubber/heatmap — not a fourth graphics axis in the 3D canvas. See{' '}
            <Link to="/lab" className="underline hover:text-foreground">
              Universe Lab
            </Link>
            .
          </>
        ),
      },
    ],
  },
  {
    title: 'Quench dynamics & particles',
    description: 'How localized excitations are created and tracked.',
    entries: [
      {
        term: 'Quench',
        body: (
          <>
            Sudden change from a simple initial state: typically product{' '}
            <M>|0…0⟩</M> with one flipped spin (defect) at the centre, then
            unitary evolution under <M>H</M>.
          </>
        ),
      },
      {
        term: 'Defect',
        body: (
          <>
            A single <M>|1⟩</M> in an otherwise <M>|0…0⟩</M> product state at
            the lattice centre. Seeds a propagating disturbance.
          </>
        ),
      },
      {
        term: 'Reference-subtracted signal',
        body: (
          <>
            <M>signalᵢ = |⟨Zᵢ⟩_defect − ⟨Zᵢ⟩_reference|</M>, where the
            reference is <M>|0…0⟩</M> evolved in lockstep. Isolates the
            propagating perturbation from uniform on-site precession caused by
            the transverse field.
          </>
        ),
      },
      {
        term: 'Worldline',
        body: (
          <>
            Track of peak signal across clock slices — a particle-like path
            through emergent space or on the lattice heatmap. Used in scattering
            and Universe Lab (purple trail).
          </>
        ),
      },
      {
        term: 'Scattering',
        body: (
          <>
            Two-defect quench: two flipped spins evolve and interact. Worldline
            tracking follows each lump; overlap and separation are measured in
            falsification tests.
          </>
        ),
      },
    ],
  },
  {
    title: 'Holography & diagnostics',
    description: 'Entropy scaling and baby RT tests — honest discrete analogues.',
    entries: [
      {
        term: 'Area law',
        body: (
          <>
            Region entropy <M>S(A)</M> scales with boundary size, not volume, in
            gapped ground states. Contrasts with Haar-random states (volume /
            Page curve).
          </>
        ),
      },
      {
        term: 'Baby Ryu–Takayanagi (RT) relation',
        body: (
          <>
            Discrete identity for redundancy-constrained states:{' '}
            <M>{'S_A ≈ ½ Σ_{a∈A, b∉A} I(a:b)'}</M>. Entropy of a region tracks
            entanglement across its boundary, not bulk volume. Slope ≈ 1 on TFIM
            ground states; breaks down under strong excitations.
          </>
        ),
      },
      {
        term: 'Mass injection (RT demo)',
        body: (
          <>
            Local <M>X</M>-flip at the centre plus brief evolution — a
            concentrated excitation that deforms the RT slope upward. Not a claim
            about literal scalar curvature.
          </>
        ),
      },
      {
        term: 'Falsification battery',
        body: (
          <>
            Automated checks on the{' '}
            <Link to="/experiments" className="underline hover:text-foreground">
              Experiments
            </Link>{' '}
            page: blind locality recovery, MI vs lattice light-cone velocity,
            RT ratio stability, multi-clock inconsistency, and more. Failures are
            research signals, not theorem violations.
          </>
        ),
      },
    ],
  },
  {
    title: 'Search & refinement',
    description: 'Prototypes for finding or adapting factorizations.',
    entries: [
      {
        term: 'Factorization search',
        body: (
          <>
            Given a Hamiltonian (or spectrum), search permutations of qubit labels
            that make physics look local on a line or grid. Modes: Pauli + MI, or
            spectrum-only with line-support bandwidth.
          </>
        ),
      },
      {
        term: 'Refinement',
        body: (
          <>
            Adaptive pressure to increase factor count when diagnostics (emergent
            dim, RT fit, etc.) fail on the current labeling — a prototype for
            dynamical Hilbert-space growth.
          </>
        ),
      },
      {
        term: 'Decoherence demo',
        body: (
          <>
            Minimal environment coupling: object + apparatus + environment
            factors entangle, producing branch-resolved tracks that look
            classical within each branch.
          </>
        ),
      },
      {
        term: 'QECC / excitation subspace probe',
        body: (
          <>
            Branch-resolved check whether localized excitations occupy a
            low-dimensional, code-like subspace — a proto test for effective
            field degrees of freedom.
          </>
        ),
      },
    ],
  },
  {
    title: 'Simulator conventions',
    description: 'How the reference engine and UI are organized.',
    entries: [
      {
        term: 'Reference engine (WASM)',
        body: (
          <>
            All heavy numerics live in{' '}
            <M>wasm/mad-dog-sim/</M>, called from the UI via a Web Worker. No
            TypeScript fallback for state-vector evolution.
          </>
        ),
      },
      {
        term: 'sim-check',
        body: (
          <>
            Regression script (<M>node scripts/sim-check.ts</M>) for emergent dim,
            light-cone velocity, RT slope, energy drift, etc. Numerical claims
            should pass here before trusting UI demos.
          </>
        ),
      },
      {
        term: 'Working learnings (docs/)',
        body: (
          <>
            Research notes under <M>docs/</M> record what the code does and what
            we measured — not claims of new physics. Start at{' '}
            <M>docs/README.md</M>.
          </>
        ),
      },
    ],
  },
]

function Glossary() {
  return (
    <div className="flex-1">
      <header className="border-b">
        <div className="mx-auto max-w-4xl px-6 py-12">
          <h1 className="text-3xl font-semibold tracking-tight mb-3">
            Glossary
          </h1>
          <p className="text-muted-foreground max-w-2xl leading-relaxed">
            Common terms used across the essay, experiments, and{' '}
            <M>docs/</M> research notes. Definitions describe how{' '}
            <em>this repository</em> uses each idea, not general textbook
            coverage.
          </p>
        </div>
      </header>

      <main className="mx-auto max-w-4xl px-6 py-12 space-y-12">
        {sections.map((section) => (
          <section key={section.title} className="space-y-4">
            <div>
              <h2 className="text-xl font-semibold">{section.title}</h2>
              {section.description && (
                <p className="text-sm text-muted-foreground mt-1">
                  {section.description}
                </p>
              )}
            </div>
            <div className="space-y-3">
              {section.entries.map((entry) => (
                <Card key={entry.term}>
                  <CardHeader className="pb-2">
                    <CardTitle className="text-base">{entry.term}</CardTitle>
                  </CardHeader>
                  <CardContent className="text-sm text-muted-foreground leading-relaxed">
                    {entry.body}
                  </CardContent>
                </Card>
              ))}
            </div>
          </section>
        ))}

        <Card className="border-dashed">
          <CardHeader>
            <CardTitle className="text-base">More detail</CardTitle>
            <CardDescription>
              Longer notes with measured results and code pointers
            </CardDescription>
          </CardHeader>
          <CardContent className="text-sm text-muted-foreground space-y-2">
            <p>
              See the <M>docs/</M> folder in the repository:{' '}
              <M>emergent-space.md</M>, <M>emergent-time.md</M>,{' '}
              <M>holography.md</M>, <M>parameters-and-phases.md</M>, and the
              index at <M>docs/README.md</M>.
            </p>
            <p>
              Interactive checks:{' '}
              <Link to="/experiments" className="underline hover:text-foreground">
                Experiments
              </Link>
              ,{' '}
              <Link to="/lab" className="underline hover:text-foreground">
                Universe Lab
              </Link>
              .
            </p>
          </CardContent>
        </Card>
      </main>
    </div>
  )
}

export default Glossary
