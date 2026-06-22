import { Separator } from '@/components/ui/separator'
import { Simulator } from '@/components/Simulator'
import { SpacetimeViz } from '@/components/SpacetimeViz'
import { Spacetime2DViz } from '@/components/Spacetime2DViz'
import { RelationalTimeViz } from '@/components/RelationalTimeViz'
import { MultiClockViz } from '@/components/MultiClockViz'
import { SimultaneityViz } from '@/components/SimultaneityViz'
import { HolographyViz } from '@/components/HolographyViz'
import { RtMassViz } from '@/components/RtMassViz'
import { RefinementViz } from '@/components/RefinementViz'
import { AdaptiveRefinementViz } from '@/components/AdaptiveRefinementViz'
import { FactorizationViz } from '@/components/FactorizationViz'
import { FactorizationRefinementViz } from '@/components/FactorizationRefinementViz'
import { LightConeCompareViz } from '@/components/LightConeCompareViz'
import { ModularClockViz } from '@/components/ModularClockViz'
import { ScatteringViz } from '@/components/ScatteringViz'
import { DecoherenceQuenchViz } from '@/components/DecoherenceQuenchViz'
import { ExcitationSubspaceViz } from '@/components/ExcitationSubspaceViz'
import { GeometryStabilityViz } from '@/components/GeometryStabilityViz'
import { FalsificationViz } from '@/components/FalsificationViz'

function Experiments() {
  return (
    <div className="flex-1">
      <header className="border-b">
        <div className="mx-auto max-w-4xl px-6 py-12">
          <h1 className="text-3xl font-semibold tracking-tight mb-3">
            Experiments
          </h1>
          <p className="text-muted-foreground max-w-2xl leading-relaxed">
            Interactive checks against the Mad-Dog program — real state-vector
            evolution, emergent geometry, Page&ndash;Wootters time, and baby
            holographic tests. Each panel runs the reference engine (optionally
            accelerated via WebAssembly).
          </p>
        </div>
      </header>

      <main className="mx-auto max-w-4xl px-6 py-12 space-y-16">
        <section className="space-y-6">
          <div>
            <h2 className="text-2xl font-semibold mb-2">Emergence, Simulated</h2>
            <p className="text-muted-foreground">
              Find a low-energy state of a Hamiltonian, measure entanglement
              between qubits, and ask whether a spatial geometry emerges from
              that entanglement alone.
            </p>
          </div>
          <Simulator />
        </section>

        <Separator />

        <section className="space-y-6">
          <div>
            <h2 className="text-2xl font-semibold mb-2">
              Spacetime from a Timeless State
            </h2>
            <p className="text-muted-foreground">
              A single timeless global state, conditioned on an internal clock,
              recovers dynamics — and the emergent geometry evolves across clock
              readings into a spacetime with a causal light cone.
            </p>
          </div>
          <SpacetimeViz />
          <Spacetime2DViz />
          <LightConeCompareViz />
          <ModularClockViz />
          <ScatteringViz />
          <DecoherenceQuenchViz />
          <ExcitationSubspaceViz />
          <GeometryStabilityViz />
          <RelationalTimeViz />
          <MultiClockViz />
          <SimultaneityViz />
        </section>

        <Separator />

        <section className="space-y-6">
          <div>
            <h2 className="text-2xl font-semibold mb-2">
              A Holographic Aside, Tested
            </h2>
            <p className="text-muted-foreground">
              Area laws, a discrete Ryu&ndash;Takayanagi relation, mass
              deformation of the RT slope, and a prototype adaptive-refinement
              diagnostic — each honestly checked, not asserted.
            </p>
          </div>
          <HolographyViz />
          <RtMassViz />
          <RefinementViz />
          <AdaptiveRefinementViz />
          <FactorizationViz />
          <FactorizationRefinementViz />
        </section>

        <Separator />

        <section className="space-y-6">
          <div>
            <h2 className="text-2xl font-semibold mb-2">Falsification Tests</h2>
            <p className="text-muted-foreground">
              Automated checks for claims specific to the Mad-Dog emergence
              program — what would disprove them if they failed.
            </p>
          </div>
          <FalsificationViz />
        </section>
      </main>
    </div>
  )
}

export default Experiments
