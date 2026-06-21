import { useCallback, useState } from 'react'
import { Play, Loader2 } from 'lucide-react'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  runRefinementNCompareAsync,
  runRefinementQuenchAsync,
  type RefinementNCompareResultWithBackend,
  type RefinementQuenchResultWithBackend,
} from '@/sim/runner-async'
import { DiagnosticTracesChart } from '@/components/refinement/DiagnosticTracesChart'

const PRESSURE = 'oklch(0.72 0.18 25)'
const RELIEF = 'oklch(0.72 0.14 155)'

function DiagnosticsTable({
  label,
  d,
}: {
  label: string
  d: RefinementQuenchResultWithBackend['slices'][0]['diagnostics']
}) {
  return (
    <div className="rounded-md border p-3 space-y-2 text-sm">
      <div className="flex items-center justify-between gap-2">
        <span className="font-medium">{label}</span>
        <Badge variant={d.needsRefinement ? 'destructive' : 'secondary'}>
          {d.needsRefinement ? 'refine?' : 'OK'}
        </Badge>
      </div>
      <dl className="grid grid-cols-2 gap-x-4 gap-y-1 text-muted-foreground">
        <dt>pressure</dt>
        <dd className="text-foreground tabular-nums">{d.pressure.toFixed(3)}</dd>
        <dt>RT slope</dt>
        <dd className="text-foreground tabular-nums">{d.rtSlope.toFixed(3)}</dd>
        <dt>RT R²</dt>
        <dd className="text-foreground tabular-nums">{d.rtR2.toFixed(3)}</dd>
        <dt>emergent dim</dt>
        <dd className="text-foreground tabular-nums">{d.emergentDim}</dd>
        <dt>area growth</dt>
        <dd className="text-foreground tabular-nums">{d.areaGrowth.toFixed(2)}</dd>
      </dl>
      {d.needsRefinement && (
        <p className="text-xs text-muted-foreground">
          Flags:{' '}
          {[
            d.reasons.rtFit && 'RT fit',
            d.reasons.rtSlope && 'RT slope',
            d.reasons.areaLaw && 'area law',
            d.reasons.emergentDim && 'emergent dim',
          ]
            .filter(Boolean)
            .join(', ')}
        </p>
      )}
    </div>
  )
}

function ComparePanel({ compare }: { compare: RefinementNCompareResultWithBackend }) {
  return (
    <div className="space-y-3">
      <div className="grid sm:grid-cols-2 gap-3">
        <DiagnosticsTable label={`n = ${compare.n}`} d={compare.small} />
        <DiagnosticsTable label={`n = ${compare.nLarge}`} d={compare.large} />
      </div>
      <p className="text-sm text-muted-foreground">
        After {compare.quenchStep} quench steps (dt = {compare.dt}), pressure at{' '}
        <span className="text-foreground">n = {compare.nLarge}</span> is{' '}
        <span
          className="tabular-nums font-medium"
          style={{ color: compare.largerRelieves ? RELIEF : PRESSURE }}
        >
          {compare.large.pressure.toFixed(3)}
        </span>{' '}
        vs{' '}
        <span className="tabular-nums font-medium text-foreground">
          {compare.small.pressure.toFixed(3)}
        </span>{' '}
        at n = {compare.n}.
        {compare.largerRelieves
          ? ' Larger factor count relieves holographic stress — consistent with adaptive refinement.'
          : ' At this quench depth the larger chain does not clearly win; try longer quench or different n.'}
      </p>
    </div>
  )
}

export function RefinementViz() {
  const [quench, setQuench] = useState<RefinementQuenchResultWithBackend | null>(
    null,
  )
  const [compare, setCompare] =
    useState<RefinementNCompareResultWithBackend | null>(null)
  const [loading, setLoading] = useState(false)

  const run = useCallback(() => {
    setLoading(true)
    const quenchConfig = {
      n: 10,
      field: 1.5,
      dt: 0.2,
      steps: 18,
      seed: 7711,
    }
    const compareConfig = {
      n: 10,
      deltaN: 2,
      field: 1.5,
      dt: 0.2,
      quenchStep: 12,
      seed: 7711,
    }
    void Promise.all([
      runRefinementQuenchAsync(quenchConfig),
      runRefinementNCompareAsync(compareConfig),
    ])
      .then(([q, c]) => {
        setQuench(q)
        setCompare(c)
      })
      .finally(() => setLoading(false))
  }, [])

  const last = quench?.slices[quench.slices.length - 1]?.diagnostics
  const first = quench?.slices[0]?.diagnostics

  return (
    <Card>
      <CardHeader>
        <CardTitle>Adaptive factor refinement (prototype)</CardTitle>
        <CardDescription>
          Track holographic consistency — RT fit, area-law proxy, emergent
          dimension — as a defect quench grows entanglement. Rising pressure
          flags when a fixed n may be inadequate; compare n vs n+2 after the
          same quench.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <Button onClick={run} disabled={loading} size="sm">
          {loading ? (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          ) : (
            <Play className="mr-2 h-4 w-4" />
          )}
          Run refinement diagnostic
        </Button>

        {quench && last && first && (
          <>
            <div className="flex flex-wrap gap-2 text-xs">
              <Badge variant="outline">
                {quench.n}-site chain, h = {quench.field}
              </Badge>
              <Badge variant="outline">
                {quench.elapsedMs.toFixed(0)} ms · {quench.backend}
              </Badge>
              <Badge
                variant={last.needsRefinement ? 'destructive' : 'secondary'}
              >
                vacuum pressure {first.pressure.toFixed(2)} →{' '}
                {last.pressure.toFixed(2)}
              </Badge>
            </div>

            <DiagnosticTracesChart slices={quench.slices} />
            <DiagnosticsTable label={`final t = ${last ? quench.slices[quench.slices.length - 1].t.toFixed(2) : ''}`} d={last} />

            {compare && (
              <>
                <h4 className="text-sm font-medium">n vs n+Δ comparison</h4>
                <ComparePanel compare={compare} />
              </>
            )}
          </>
        )}
      </CardContent>
    </Card>
  )
}
