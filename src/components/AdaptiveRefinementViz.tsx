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
  runAdaptiveRefinementAsync,
  type AdaptiveRefinementResultWithBackend,
} from '@/sim/runner-async'
import { DiagnosticTracesChart } from '@/components/refinement/DiagnosticTracesChart'
import type { RefinementDiagnostics } from '@/sim/types'

const RELIEF = 'oklch(0.72 0.14 155)'
const PRESSURE = 'oklch(0.72 0.18 25)'

function DiagnosticsTable({
  label,
  d,
}: {
  label: string
  d: RefinementDiagnostics
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
      </dl>
    </div>
  )
}

function SplitPanel({ result }: { result: AdaptiveRefinementResultWithBackend }) {
  const ev = result.splitEvent
  if (!ev) {
    return (
      <p className="text-sm text-muted-foreground">
        No split trigger during this quench — holographic diagnostics never crossed
        the refinement threshold. Try longer evolution or higher entanglement growth.
      </p>
    )
  }

  return (
    <div className="space-y-3">
      <div className="flex flex-wrap gap-2 text-xs">
        <Badge variant="outline">
          trigger step {ev.triggerStep} (t = {ev.triggerT.toFixed(2)})
        </Badge>
        <Badge variant="outline">
          split hint site {ev.suggestedSplitSite}
        </Badge>
        <Badge variant={ev.accepted ? 'default' : 'destructive'}>
          split {ev.accepted ? 'accepted' : 'rejected'}
        </Badge>
      </div>
      <div className="grid sm:grid-cols-2 gap-3">
        <DiagnosticsTable label={`n = ${ev.preN} (before)`} d={ev.pre} />
        <DiagnosticsTable label={`n = ${ev.postN} (after +${result.deltaN})`} d={ev.post} />
      </div>
      <p className="text-sm text-muted-foreground">
        At the trigger step, pressure at{' '}
        <span className="text-foreground">n = {ev.postN}</span> is{' '}
        <span
          className="tabular-nums font-medium"
          style={{ color: ev.accepted ? RELIEF : PRESSURE }}
        >
          {ev.post.pressure.toFixed(3)}
        </span>{' '}
        vs{' '}
        <span className="tabular-nums font-medium text-foreground">
          {ev.pre.pressure.toFixed(3)}
        </span>{' '}
        at n = {ev.preN} (Δ = {ev.pressureDelta.toFixed(3)}).
        {ev.accepted
          ? ' Adding factors relieves holographic stress at the same quench depth — consistent with adaptive mesh refinement.'
          : ' At this depth a larger chain did not clearly help; the heuristic would reject the split.'}
      </p>
      <p className="text-xs text-muted-foreground leading-relaxed">
        Honest limit: we compare independent quenches on n vs n+Δ chains, not an
        in-place tensor factor split of the same |ψ⟩.
      </p>
    </div>
  )
}

export function AdaptiveRefinementViz() {
  const [result, setResult] = useState<AdaptiveRefinementResultWithBackend | null>(
    null,
  )
  const [loading, setLoading] = useState(false)

  const run = useCallback(() => {
    setLoading(true)
    void runAdaptiveRefinementAsync({
      n: 10,
      field: 1.5,
      dt: 0.2,
      steps: 18,
      seed: 7711,
      deltaN: 2,
    })
      .then(setResult)
      .finally(() => setLoading(false))
  }, [])

  const last = result?.slices[result.slices.length - 1]?.diagnostics
  const first = result?.slices[0]?.diagnostics

  return (
    <Card>
      <CardHeader>
        <CardTitle>Adaptive holographic refinement</CardTitle>
        <CardDescription>
          Monitor RT fit, area-law proxy, and emergent dimension under a defect
          quench. When composite pressure crosses the refinement threshold, apply a
          toy split rule: compare diagnostics at n vs n+Δ at the same quench depth.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <Button onClick={run} disabled={loading} size="sm">
          {loading ? (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          ) : (
            <Play className="mr-2 h-4 w-4" />
          )}
          Run adaptive refinement
        </Button>

        {result && last && first && (
          <>
            <div className="flex flex-wrap gap-2 text-xs">
              <Badge variant="outline">
                {result.n}-site → {result.n + result.deltaN} split, h = {result.field}
              </Badge>
              <Badge variant="outline">
                {result.elapsedMs.toFixed(0)} ms · {result.backend}
              </Badge>
              <Badge variant="outline">
                peak pressure {result.peakPressure.toFixed(2)} @ step {result.peakStep}
              </Badge>
              <Badge
                variant={last.needsRefinement ? 'destructive' : 'secondary'}
              >
                vacuum {first.pressure.toFixed(2)} → {last.pressure.toFixed(2)}
              </Badge>
            </div>

            <DiagnosticTracesChart
              slices={result.slices}
              triggerStep={result.splitEvent?.triggerStep}
            />
            {result.splitEvent && (
              <p className="text-xs text-muted-foreground -mt-2">
                Dashed vertical line = first step where refinement is requested.
              </p>
            )}

            <h4 className="text-sm font-medium">Split decision</h4>
            <SplitPanel result={result} />
          </>
        )}
      </CardContent>
    </Card>
  )
}
