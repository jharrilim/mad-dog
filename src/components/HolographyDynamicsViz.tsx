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
  runCurvatureProxyQuenchAsync,
  runPredictiveRefinementAsync,
  runRtQuenchAsync,
  type CurvatureProxyQuenchResultWithBackend,
  type PredictiveRefinementResultWithBackend,
  type RtQuenchResultWithBackend,
} from '@/sim/runner-async'

const RT = 'oklch(0.72 0.16 290)'
const DENSITY = 'oklch(0.78 0.13 60)'
const GEO = 'oklch(0.72 0.14 155)'

function TimeSeriesChart({
  slices,
  lines,
}: {
  slices: { t: number; step: number }[]
  lines: { key: string; color: string; values: number[]; label: string }[]
}) {
  if (slices.length === 0) return null
  const W = 460
  const H = 220
  const pad = { l: 44, r: 16, t: 16, b: 32 }
  const plotW = W - pad.l - pad.r
  const plotH = H - pad.t - pad.b
  const maxT = slices[slices.length - 1]?.t ?? 1
  const allVals = lines.flatMap((l) => l.values)
  const maxY = Math.max(0.01, ...allVals)
  const x = (t: number) => pad.l + (t / maxT) * plotW
  const y = (v: number) => pad.t + plotH - (v / maxY) * plotH

  return (
    <svg viewBox={`0 0 ${W} ${H}`} className="w-full h-auto" role="img">
      <rect x={pad.l} y={pad.t} width={plotW} height={plotH} fill="none" stroke="currentColor" strokeOpacity={0.15} />
      {lines.map((line) => (
        <polyline
          key={line.key}
          fill="none"
          stroke={line.color}
          strokeWidth={2}
          points={slices.map((s, i) => `${x(s.t)},${y(line.values[i] ?? 0)}`).join(' ')}
        />
      ))}
      <text x={pad.l} y={H - 8} className="fill-muted-foreground text-[10px]">
        t
      </text>
      <g className="fill-muted-foreground text-[10px]">
        {lines.map((line, i) => (
          <text key={line.key} x={pad.l + i * 120} y={12} fill={line.color}>
            {line.label}
          </text>
        ))}
      </g>
    </svg>
  )
}

const DEFAULT = { n: 10, field: 1.5, dt: 0.2, steps: 16, seed: 7711 }

export function HolographyDynamicsViz() {
  const [rt, setRt] = useState<RtQuenchResultWithBackend | null>(null)
  const [predict, setPredict] = useState<PredictiveRefinementResultWithBackend | null>(null)
  const [proxy, setProxy] = useState<CurvatureProxyQuenchResultWithBackend | null>(null)
  const [loading, setLoading] = useState(false)

  const run = useCallback(() => {
    setLoading(true)
    void Promise.all([
      runRtQuenchAsync(DEFAULT),
      runPredictiveRefinementAsync({ ...DEFAULT, steps: 18, deltaN: 2 }),
      runCurvatureProxyQuenchAsync({ ...DEFAULT, xi: 1.0 }),
    ])
      .then(([rtRes, predRes, proxyRes]) => {
        setRt(rtRes)
        setPredict(predRes)
        setProxy(proxyRes)
      })
      .finally(() => setLoading(false))
  }, [])

  return (
    <Card>
      <CardHeader>
        <CardTitle>Holography under dynamics (S5)</CardTitle>
        <CardDescription>
          RT time series during defect quench, predictive split early warning, and geodesic
          deviation as a second curvature proxy.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <Button onClick={run} disabled={loading} size="sm">
          {loading ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <Play className="mr-2 h-4 w-4" />}
          Run quench diagnostics
        </Button>

        {rt && (
          <div className="space-y-2">
            <div className="flex items-center gap-2">
              <span className="text-sm font-medium">RT vs density</span>
              <Badge variant={rt.structuredDeviation ? 'secondary' : 'destructive'}>
                ρ={rt.deviationDensityCorr.toFixed(2)}
              </Badge>
            </div>
            <TimeSeriesChart
              slices={rt.slices}
              lines={[
                { key: 'slope', color: RT, values: rt.slices.map((s) => s.slopeDeviation), label: 'slope deficit' },
                { key: 'density', color: DENSITY, values: rt.slices.map((s) => s.areaPressure), label: 'area pressure' },
              ]}
            />
          </div>
        )}

        {predict && (
          <div className="rounded-md border p-3 text-sm space-y-1">
            <div className="font-medium">Predictive refinement</div>
            <p className="text-muted-foreground">
              Early warning step {predict.earlyWarningStep ?? '—'} → failure {predict.failureStep ?? '—'}
              {' '}(lead {predict.leadTime} steps)
            </p>
            <Badge variant={predict.lateSplitRecoverable ? 'secondary' : 'outline'}>
              {predict.lateSplitRecoverable ? 'late split relieves pressure' : 'split pending'}
            </Badge>
          </div>
        )}

        {proxy && (
          <div className="space-y-2">
            <div className="flex items-center gap-2">
              <span className="text-sm font-medium">Curvature proxy suite</span>
              <Badge variant={proxy.internallyConsistent ? 'secondary' : 'destructive'}>
                geoρ={proxy.geoDensityCorr.toFixed(2)}
              </Badge>
            </div>
            <TimeSeriesChart
              slices={proxy.slices}
              lines={[
                { key: 'geo', color: GEO, values: proxy.slices.map((s) => s.geodesicDeviation), label: 'geodesic dev' },
                { key: 'slope', color: RT, values: proxy.slices.map((s) => s.rtSlopeDeviation), label: 'RT slope dev' },
              ]}
            />
          </div>
        )}
      </CardContent>
    </Card>
  )
}
