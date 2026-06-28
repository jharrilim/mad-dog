import { Link } from 'react-router-dom'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Separator } from '@/components/ui/separator'

function M({ children }: { children: React.ReactNode }) {
  return (
    <code className="font-mono text-sm bg-muted px-1.5 py-0.5 rounded">
      {children}
    </code>
  )
}

const signatureRows = [
  {
    id: 'S1',
    theme: 'Locality entanglement-readable',
    examples: 'A, A′, K, M, AA, AL, AK',
    failure: 'Blind inference cannot recover the native lattice labeling — or negative controls falsely pass.',
  },
  {
    id: 'S2',
    theme: 'Low-dimensional MI geometry',
    examples: 'B, J, J′',
    failure: 'Mutual-information distances do not track lattice separation or stay stable under quench.',
  },
  {
    id: 'S3',
    theme: 'Relational time (no global clock)',
    examples: 'D, D′, G, G′, N, AJ, AM',
    failure: 'Observer clocks collapse to a single uniform time — or the signature never breaks at scale.',
  },
  {
    id: 'S4',
    theme: 'Causal light cones',
    examples: 'E, Q, AD, AE',
    failure: 'Excitations do not propagate with cone-like fronts or stable exchange phases.',
  },
  {
    id: 'S5',
    theme: 'Holographic entanglement',
    examples: 'C, C′, F, F′, H, R′, AC',
    failure: 'Area-law / RT proxies fail, or refinement pressure does not track entanglement structure.',
  },
  {
    id: 'S6',
    theme: 'Particles = excitations',
    examples: 'E, S′',
    failure: 'Defects are not localized or do not live longer in the ordered phase.',
  },
  {
    id: 'S7',
    theme: 'Classical branches',
    examples: 'I, T′',
    failure: 'Branch Born weights do not track distinguishability after minimal env coupling.',
  },
  {
    id: 'S8',
    theme: 'IR subspace / code-like',
    examples: 'I, I′, U′, X, AI',
    failure: 'No stabilizer or code-like structure on branch excitation windows.',
  },
  {
    id: 'S9',
    theme: 'Lorentz-ish IR causality',
    examples: 'L, L′, O, P, Y, Z, AF',
    failure: 'Cardinal front speeds, dispersion, or multi-frame boost tests are strongly anisotropic.',
  },
  {
    id: 'S10',
    theme: 'Factor count not arbitrary',
    examples: 'F, H, V′, W′, AB',
    failure: 'Adaptive refinement or holographic bound probes show no pressure-linked factor dynamics.',
  },
] as const

const categoryRows = [
  {
    category: 'Blind locality & factorization',
    tests: 'A, A′, K, AA, AL, AG, AH, AK',
    role: 'Can we recover lattice structure without consulting native Ĥ labeling?',
  },
  {
    category: 'Negative controls',
    tests: 'M',
    role: 'Random and scrambled-spectrum chains must not recover — guards against score hacks.',
  },
  {
    category: 'Emergent geometry',
    tests: 'B, J, J′',
    role: 'MI-MDS geometry matches lattice distances and stays gauge-free stable.',
  },
  {
    category: 'Relational & multi-clock time',
    tests: 'D, D′, G, G′, N, AJ, AM',
    role: 'No global Δt; defect clocks sync locally but networks disagree; AM checks scaling boundary.',
  },
  {
    category: 'Causality & scattering',
    tests: 'E, L, L′, O, P, Q, AD, AE, AF, Y, Z',
    role: 'Light cones, dispersion, boost proxies, and two-defect exchange on chain/grid.',
  },
  {
    category: 'Holography & refinement',
    tests: 'C, F, H, R′, F′, C′, V′, W′, AC',
    role: 'RT ratios, adaptive splits, hold-out diagnostic self-consistency, phase-dependent emergence.',
  },
  {
    category: 'Matter & classicality',
    tests: 'I, I′, S′, T′, U′, X, AI',
    role: 'Branch-resolved excitations, stabilizers, Born weights, EFT DOF agreement, QECC probes.',
  },
] as const

function Falsification() {
  return (
    <div className="flex-1">
      <header className="border-b">
        <div className="mx-auto max-w-4xl px-6 py-12">
          <h1 className="text-3xl font-semibold tracking-tight mb-3">
            Falsification Battery
          </h1>
          <p className="text-muted-foreground max-w-2xl leading-relaxed">
            How we test Mad-Dog-specific emergence claims — not generic gapped-system
            facts — with an automated suite you can run in the browser or from the
            command line. The pattern is meant to be portable to larger simulators
            and real QPU programs.
          </p>
        </div>
      </header>

      <main className="mx-auto max-w-4xl px-6 py-12 space-y-16">
        <section className="space-y-4 text-muted-foreground leading-relaxed">
          <h2 className="text-2xl font-semibold text-foreground">What this is</h2>
          <p>
            Mad-Dog Everettianism predicts that space, time, particles, and
            classicality emerge from entanglement on a suitable factorization of a
            single quantum state. That is a research program, not a finished theorem.
            The <strong className="text-foreground">falsification battery</strong> is
            our attempt to spell out — in executable form — what would{' '}
            <em>disprove</em> Mad-Dog-specific signatures if they failed, while
            separating those from observations any local gapped chain might satisfy
            (ground-state area laws, approximate RT on special states, and so on).
          </p>
          <p>
            Each test has a letter ID (<M>A</M> through <M>AM</M> in the current
            suite, ~45 checks), a named claim, a numeric pass criterion, and a
            one-line detail string. The battery runs entirely in Rust compiled to
            WASM; TypeScript only invokes it and displays results.
          </p>
        </section>

        <Separator />

        <section className="space-y-4">
          <h2 className="text-2xl font-semibold">Mad-Dog signatures S1–S10</h2>
          <p className="text-muted-foreground leading-relaxed">
            The roadmap organizes emergence claims into ten operational signatures.
            Multiple battery tests target each signature; passing the suite means
            these behaviors show up on models we did not hand-tune, with negative
            controls failing loudly.
          </p>
          <div className="overflow-x-auto rounded-md border">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b bg-muted/50 text-left">
                  <th className="px-4 py-3 font-medium">Sig.</th>
                  <th className="px-4 py-3 font-medium">Theme</th>
                  <th className="px-4 py-3 font-medium">Example tests</th>
                  <th className="px-4 py-3 font-medium">What failure would mean</th>
                </tr>
              </thead>
              <tbody className="text-muted-foreground">
                {signatureRows.map((row) => (
                  <tr key={row.id} className="border-b last:border-0">
                    <td className="px-4 py-3 font-mono text-foreground">{row.id}</td>
                    <td className="px-4 py-3 text-foreground">{row.theme}</td>
                    <td className="px-4 py-3 font-mono text-xs">{row.examples}</td>
                    <td className="px-4 py-3">{row.failure}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>

        <Separator />

        <section className="space-y-6">
          <h2 className="text-2xl font-semibold">Battery philosophy</h2>
          <div className="grid gap-4 md:grid-cols-1">
            <Card>
              <CardHeader>
                <CardTitle className="text-base">Generic vs Mad-Dog-specific</CardTitle>
              </CardHeader>
              <CardContent className="text-sm text-muted-foreground leading-relaxed space-y-2">
                <p>
                  A ground-state area law or RT slope near unity on a TFIM chain is
                  largely generic QI. Tests like <M>C</M> (RT ratio stability) or{' '}
                  <M>B</M> (MI vs lattice light-cone velocity) target tighter
                  Mad-Dog-shaped claims. When in doubt, we ask: would this still
                  pass on a random nonlocal Hamiltonian or a scrambled spectrum?
                </p>
              </CardContent>
            </Card>
            <Card>
              <CardHeader>
                <CardTitle className="text-base">Negative controls</CardTitle>
              </CardHeader>
              <CardContent className="text-sm text-muted-foreground leading-relaxed space-y-2">
                <p>
                  Test <M>M</M> runs factorization search on a random nonlocal chain
                  and a scrambled-spectrum shuffled chain. Both must{' '}
                  <em>fail</em> to recover identity. Without this, a permissive
                  scoring function could make everything look Mad-Dog-ly.
                </p>
              </CardContent>
            </Card>
            <Card>
              <CardHeader>
                <CardTitle className="text-base">Hold-out grids (prototype diagnostics)</CardTitle>
              </CardHeader>
              <CardContent className="text-sm text-muted-foreground leading-relaxed space-y-2">
                <p>
                  Several <M>′</M> tests (<M>F′</M>, <M>V′</M>, <M>R′</M>,{' '}
                  <M>C′</M>, <M>T′</M>, <M>I′</M>) verify{' '}
                  <strong className="text-foreground">diagnostic self-consistency</strong>,
                  not independent physics. Thresholds are tuned on a calibration demo
                  (e.g. <M>n=10, h=1.5, seed=7711</M>); pass/fail is evaluated on a
                  separate hold-out <M>(n, field, seed)</M> grid defined in Rust (
                  <M>refinement_holdout.rs</M>, <M>matter_holdout.rs</M>). Treat
                  these as regression guards on the refinement stack, not proofs of
                  emergent gravity.
                </p>
              </CardContent>
            </Card>
            <Card>
              <CardHeader>
                <CardTitle className="text-base">Scaling boundaries</CardTitle>
              </CardHeader>
              <CardContent className="text-sm text-muted-foreground leading-relaxed space-y-2">
                <p>
                  Some signatures should strengthen with system size; others should
                  eventually break. Test <M>L′</M> checks that cardinal front-speed
                  coefficient of variation improves from 3×3 to 4×4 grids. Test{' '}
                  <M>AM</M> encodes the opposite: multi-clock test <M>G</M> passes at{' '}
                  <M>n=9</M> but fails at <M>n=11</M>, marking where relational-time
                  structure stops looking Mad-Dog-like on long chains. Test{' '}
                  <M>AL</M> extends blind MI+bandwidth recovery to 2D grid and torus
                  targets, not just 1D chains.
                </p>
              </CardContent>
            </Card>
          </div>
        </section>

        <Separator />

        <section className="space-y-4">
          <h2 className="text-2xl font-semibold">Tests by category</h2>
          <p className="text-muted-foreground leading-relaxed">
            Grouped summary — not every letter ID. Full pass criteria live in the
            repository&apos;s <M>docs/falsification.md</M>.
          </p>
          <div className="overflow-x-auto rounded-md border">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b bg-muted/50 text-left">
                  <th className="px-4 py-3 font-medium">Category</th>
                  <th className="px-4 py-3 font-medium">Tests</th>
                  <th className="px-4 py-3 font-medium">Role</th>
                </tr>
              </thead>
              <tbody className="text-muted-foreground">
                {categoryRows.map((row) => (
                  <tr key={row.category} className="border-b last:border-0">
                    <td className="px-4 py-3 text-foreground">{row.category}</td>
                    <td className="px-4 py-3 font-mono text-xs whitespace-nowrap">
                      {row.tests}
                    </td>
                    <td className="px-4 py-3">{row.role}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </section>

        <Separator />

        <section className="space-y-4 text-muted-foreground leading-relaxed">
          <h2 className="text-2xl font-semibold text-foreground">
            How it is implemented
          </h2>
          <p>
            All numerical work lives in{' '}
            <M>wasm/mad-dog-sim/src/run_falsification.rs</M>. The entry point{' '}
            <M>run_falsification_battery()</M> sequentially invokes existing runners
            (factorization search, modular clocks, RT quench, scattering, Poincaré
            composite, hold-out evaluators, etc.) and collects{' '}
            <M>FalsificationTest</M> records with id, name, pass flag, and detail
            string. The WASM export <M>run_falsification_battery_json</M> is
            registered in the sim worker and called from the UI.
          </p>
          <p>
            From the command line:
          </p>
          <pre className="rounded-md border bg-muted/40 px-4 py-3 text-sm font-mono text-foreground overflow-x-auto">
            npm run check:falsification
          </pre>
          <p>
            That script (<M>scripts/falsification-check.ts</M>) loads the WASM module
            synchronously, prints each test, and exits non-zero on any failure — suitable
            for CI. The suite includes exact factorization searches and the Phase-1
            model zoo (<M>K</M>), so expect several minutes on a laptop.
          </p>
          <p>
            Related benches (not all folded into the main battery):
          </p>
          <ul className="list-disc list-inside space-y-1 text-sm">
            <li>
              <M>npm run bench:factor</M> — factor-count dynamics (<M>AB</M>) and
              holographic phase emergence (<M>AC</M>)
            </li>
            <li>
              <M>npm run bench:factorization</M>, <M>bench:time</M>,{' '}
              <M>bench:causality</M>, <M>bench:holography</M>, <M>bench:matter</M>,{' '}
              <M>bench:scaling</M> — domain sweeps that informed individual tests
            </li>
          </ul>
          <p>
            In the browser, open{' '}
            <Link to="/experiments" className="underline hover:text-foreground">
              Experiments → Falsification Tests
            </Link>{' '}
            and click <em>Run falsification battery</em> to execute the same WASM
            path asynchronously in a worker.
          </p>
        </section>

        <Separator />

        <section className="space-y-4 text-muted-foreground leading-relaxed">
          <h2 className="text-2xl font-semibold text-foreground">
            Adapting this pattern
          </h2>
          <p>
            If you are building a simulator or compiling circuits for a QPU, a
            falsification battery is a structured way to avoid fooling yourself:
          </p>
          <ol className="list-decimal list-inside space-y-3">
            <li>
              <strong className="text-foreground">Name signatures, not demos.</strong>{' '}
              Write down operational claims (recoverable locality, relational clocks,
              etc.) before wiring plots. Map each claim to at least one test with a
              numeric threshold.
            </li>
            <li>
              <strong className="text-foreground">Separate calibration from evaluation.</strong>{' '}
              Tune thresholds on one configuration; pass/fail on a hold-out grid you
              never used for tuning. Document which tests are diagnostic
              self-consistency vs independent observables.
            </li>
            <li>
              <strong className="text-foreground">Require negative controls.</strong>{' '}
              Random nonlocal models and deliberately broken inputs should fail the
              same scores that real models pass.
            </li>
            <li>
              <strong className="text-foreground">Probe scaling explicitly.</strong>{' '}
              Include tests that should improve with <M>n</M> and tests that should
              eventually break — otherwise you cannot tell success from finite-size
              coincidence.
            </li>
            <li>
              <strong className="text-foreground">Keep heavy math off the UI thread.</strong>{' '}
              Here, Rust/WASM runs the battery; the UI only displays JSON results.
              On a QPU stack, the same structure applies: compile checks as offline
              jobs, not notebook cells that silently drift.
            </li>
          </ol>
          <p>
            Thresholds in this repo are hand-tuned on TFIM demos at small{' '}
            <M>n</M>. Failures are research signals to investigate, not automatic
            refutations of Carroll &amp; Singh (2018).
          </p>
        </section>

        <Separator />

        <section className="space-y-4">
          <h2 className="text-2xl font-semibold">Honest limits</h2>
          <Card className="border-dashed">
            <CardContent className="pt-6 text-sm text-muted-foreground leading-relaxed space-y-3">
              <p>
                <strong className="text-foreground">Toy lattices.</strong> Most tests
                use open chains, 3×3 grids, or small cubes at{' '}
                <M>n ≲ 14</M> qubits. Nothing here touches continuum limits or
                experimental data.
              </p>
              <p>
                <strong className="text-foreground">Fixed factor count.</strong> The
                sim assumes <M>ℋ = ⊗ₐ ℋₐ</M> with two-level factors already present.
                Tests like <M>W′</M> and <M>AB</M> probe pressure-linked refinement
                but do not derive why there are <M>n</M> qubits in the first place.
              </p>
              <p>
                <strong className="text-foreground">Proxies, not proofs.</strong>{' '}
                Lorentz tests (<M>L</M>–<M>Z</M>) measure coefficient-of-variation
                and dispersion fits on emergent clocks — useful sanity checks, not
                representations of the Poincaré group. Holographic tests use discrete
                RT ratios and area-law slopes, not a full AdS/CFT dictionary.
              </p>
              <p>
                <strong className="text-foreground">Circularity audit.</strong> Phase 9
                hardening removed tautological pass criteria (e.g.{' '}
                <M>W′</M> now uses blind factorization on <M>|ψ⟩</M>, not native{' '}
                <M>Ĥ</M>). Remaining diagnostic tests are labeled as such in{' '}
                <M>docs/circularity-audit.md</M>.
              </p>
            </CardContent>
          </Card>
        </section>

        <Card className="border-dashed">
          <CardHeader>
            <CardTitle className="text-base">Run it / read more</CardTitle>
            <CardDescription>
              Interactive runner and repository reference
            </CardDescription>
          </CardHeader>
          <CardContent className="text-sm text-muted-foreground space-y-2">
            <p>
              <Link to="/experiments" className="underline hover:text-foreground">
                Experiments page
              </Link>{' '}
              — live WASM battery runner at the bottom of the page.
            </p>
            <p>
              Repository docs: <M>docs/falsification.md</M> (full test table),{' '}
              <M>docs/roadmap.md</M> (signature program),{' '}
              <M>docs/circularity-audit.md</M> (independence levels).
            </p>
          </CardContent>
        </Card>
      </main>
    </div>
  )
}

export default Falsification
