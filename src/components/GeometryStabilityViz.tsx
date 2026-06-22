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
  runGeometryStabilityAsync,
  type GeometryStabilityResultWithBackend,
} from '@/sim/runner-async'

function StabilityChart({ result }: { result: GeometryStabilityResultWithBackend }) {
  const W = 360
  const H = 140
  const pad = { l: 36, r: 8, t: 12, b: 24 }
  const innerW = W - pad.l - pad.r
  const innerH = H - pad.t - pad.b
  const slices = result.slices
  if (slices.length < 2) return null

  const path = (key: 'rankCorrelation' | 'distanceDrift') => {
    const pts = slices.map((s, i) => {
      const x = pad.l + (i / (slices.length - 1)) * innerW
      const v = s[key]
      const y = pad.t + innerH * (1 - Math.min(v, key === 'rankCorrelation' ? 1 : 0.6) / (key === 'rankCorrelation' ? 1 : 0.6))
      return `${i === 0 ? 'M' : 'L'} ${x} ${y}`
    })
    return pts.join(' ')
  }

  return (
    <svg viewBox={`0 0 ${W} ${H}`} className="w-full max-w-lg h-auto" role="img">
      <path
        fill="none"
        stroke="oklch(0.62 0.14 290)"
        strokeWidth={1.5}
        d={path('rankCorrelation')}
      />
      <path
        fill="none"
        stroke="oklch(0.55 0.08 250)"
        strokeWidth={1.5}
        strokeDasharray="4 3"
        d={path('distanceDrift')}
      />
      <text x={pad.l} y={H - 6} className="fill-muted-foreground text-[9px]">
        slice k →
      </text>
      <text x={W - 8} y={pad.t + 10} textAnchor="end" className="fill-muted-foreground text-[9px]">
        solid = rank ρ, dashed = ΔD
      </text>
    </svg>
  )
}

export function GeometryStabilityViz() {
  const [result, setResult] = useState<GeometryStabilityResultWithBackend | null>(null)
  const [loading, setLoading] = useState(false)

  const run = useCallback(() => {
    setLoading(true)
    void runGeometryStabilityAsync({
      n: 10,
      field: 1.2,
      dt: 0.2,
      steps: 20,
      xi: 1.0,
      seed: 42,
    })
      .then(setResult)
      .finally(() => setLoading(false))
  }, [])

  return (
    <Card>
      <CardHeader>
        <CardTitle>Gauge-free MI geometry stability</CardTitle>
        <CardDescription>
          Track mutual-information distance matrices across clock slices without
          Procrustes alignment. Spearman rank correlation measures whether relative
          qubit–qubit distances stay ordered as an excitation propagates.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <Button onClick={run} disabled={loading} size="sm">
          {loading ? (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          ) : (
            <Play className="mr-2 h-4 w-4" />
          )}
          Run stability scan
        </Button>
        {result && (
          <>
            <div className="flex flex-wrap gap-2">
              <Badge variant={result.geometryStable ? 'default' : 'secondary'}>
                {result.geometryStable ? 'Stable ranking' : 'Unstable ranking'}
              </Badge>
              <Badge variant="outline">
                mean ρ {result.meanRankCorrelation.toFixed(3)}
              </Badge>
              <Badge variant="outline">
                mean ΔD {result.meanDistanceDrift.toFixed(3)}
              </Badge>
              <Badge variant="outline">dim σ {result.dimStd.toFixed(2)}</Badge>
            </div>
            <StabilityChart result={result} />
          </>
        )}
      </CardContent>
    </Card>
  )
}
