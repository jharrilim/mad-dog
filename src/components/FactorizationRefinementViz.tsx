import { useCallback, useMemo, useState } from 'react'
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
  runFactorizationRefinementStudyAsync,
  type FactorizationRefinementResultWithBackend,
} from '@/sim/runner-async'
import { FactorizationCostBanner } from '@/components/factorization/FactorizationCostBanner'
import { DiagnosticTracesChart } from '@/components/refinement/DiagnosticTracesChart'
import { estimateFactorizationRefinementCost } from '@/sim/factorization-warnings'
import type { FactorizationRefinementConfig } from '@/sim/types'

const PRESSURE = 'oklch(0.72 0.18 25)'
const SCORE = 'oklch(0.72 0.14 155)'

function DualChart({ result }: { result: FactorizationRefinementResultWithBackend }) {
  const data = result.slices
  const W = 460
  const H = 220
  const pad = { l: 44, r: 16, t: 16, b: 38 }
  const plotW = W - pad.l - pad.r
  const plotH = H - pad.t - pad.b
  const maxT = data[data.length - 1]?.t ?? 1
  const maxP = Math.max(...data.map((d) => d.refinementPressure), 0.5)
  const maxS = Math.max(...data.map((d) => d.factorizationScore), 0.5)
  const x = (t: number) => pad.l + (t / maxT) * plotW
  const yP = (v: number) => pad.t + plotH - (v / maxP) * plotH
  const yS = (v: number) => pad.t + plotH - (v / maxS) * plotH
  const pressureLine = data.map((d) => `${x(d.t)},${yP(d.refinementPressure)}`).join(' ')
  const scoreLine = data.map((d) => `${x(d.t)},${yS(d.factorizationScore)}`).join(' ')

  return (
    <svg viewBox={`0 0 ${W} ${H}`} className="w-full h-auto" role="img" aria-label="Refinement vs factorization">
      <polyline fill="none" stroke={PRESSURE} strokeWidth={2} points={pressureLine} />
      <polyline fill="none" stroke={SCORE} strokeWidth={2} strokeDasharray="5 3" points={scoreLine} />
      <text x={pad.l + 4} y={pad.t + 12} className="fill-muted-foreground text-[9px]">solid = refinement pressure</text>
      <text x={pad.l + 4} y={pad.t + 24} className="fill-muted-foreground text-[9px]">dashed = factorization score</text>
    </svg>
  )
}

export function FactorizationRefinementViz() {
  const [result, setResult] = useState<FactorizationRefinementResultWithBackend | null>(null)
  const [loading, setLoading] = useState(false)

  const studyConfig = useMemo(
    (): FactorizationRefinementConfig => ({
      n: 10,
      field: 1.5,
      dt: 0.2,
      steps: 14,
      seed: 7711,
      annealingSteps: 600,
    }),
    [],
  )

  const costWarning = useMemo(
    () => estimateFactorizationRefinementCost(studyConfig),
    [studyConfig],
  )

  const run = useCallback(() => {
    setLoading(true)
    void runFactorizationRefinementStudyAsync(studyConfig)
      .then(setResult)
      .finally(() => setLoading(false))
  }, [studyConfig])

  const last = result?.slices[result.slices.length - 1]

  return (
    <Card>
      <CardHeader>
        <CardTitle>Quench + factorization drift</CardTitle>
        <CardDescription>
          As a defect quench grows entanglement, track holographic refinement pressure
          alongside the best line factorization for |ψ(t)⟩. Permutation drift measures
          whether the preferred labeling changes over emergent time.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <FactorizationCostBanner warning={costWarning} />
        <Button onClick={run} disabled={loading} size="sm">
          {loading ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <Play className="mr-2 h-4 w-4" />}
          Run joint study
        </Button>
        {result && last && (
          <>
            <div className="flex flex-wrap gap-2 text-xs">
              <Badge variant="outline">{result.n}-site chain</Badge>
              <Badge variant="outline">{result.elapsedMs.toFixed(0)} ms · {result.backend}</Badge>
              <Badge variant="outline">perm drift = {last.permDrift}</Badge>
            </div>
            <DualChart result={result} />
            <DiagnosticTracesChart
              slices={result.slices.map((s) => ({
                t: s.t,
                step: s.step,
                diagnostics: s.diagnostics,
              }))}
            />
            <p className="text-sm text-muted-foreground">
              Final pressure {last.refinementPressure.toFixed(2)}, factorization score{' '}
              {last.factorizationScore.toFixed(2)}, locality{' '}
              {(last.localityFraction * 100).toFixed(0)}%.
            </p>
          </>
        )}
      </CardContent>
    </Card>
  )
}
