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
  runSimultaneityAsync,
  type SimultaneityResultWithBackend,
} from '@/sim/runner-async'

function SkewChart({ result }: { result: SimultaneityResultWithBackend }) {
  const W = 460
  const H = 220
  const pad = { l: 44, r: 16, t: 16, b: 38 }
  const plotW = W - pad.l - pad.r
  const plotH = H - pad.t - pad.b
  const n = result.slices.length
  const maxSkew = Math.max(result.maxTauSkew, 1e-6)
  const x = (k: number) => pad.l + (k / Math.max(n - 1, 1)) * plotW
  const y = (v: number) => pad.t + plotH - (v / maxSkew) * plotH
  const skewLine = result.slices.map((s) => `${x(s.k)},${y(s.tauSkew)}`).join(' ')
  const tauA = result.slices.map((s) => `${x(s.k)},${y(s.tauA)}`).join(' ')
  const tauB = result.slices.map((s) => `${x(s.k)},${y(s.tauB)}`).join(' ')

  return (
    <svg viewBox={`0 0 ${W} ${H}`} className="w-full h-auto" role="img">
      <polyline fill="none" stroke="oklch(0.72 0.16 290)" strokeWidth={2} points={skewLine} />
      <polyline
        fill="none"
        stroke="oklch(0.72 0.14 155)"
        strokeWidth={1.5}
        strokeDasharray="4 3"
        opacity={0.7}
        points={tauA}
      />
      <polyline
        fill="none"
        stroke="oklch(0.78 0.13 60)"
        strokeWidth={1.5}
        strokeDasharray="4 3"
        opacity={0.7}
        points={tauB}
      />
      <text x={pad.l + 4} y={pad.t + 12} className="fill-muted-foreground text-[9px]">
        solid = |τ_A − τ_B| (site {result.clockA} vs {result.clockB})
      </text>
    </svg>
  )
}

export function SimultaneityViz() {
  const [result, setResult] = useState<SimultaneityResultWithBackend | null>(null)
  const [loading, setLoading] = useState(false)

  const run = useCallback(() => {
    setLoading(true)
    void runSimultaneityAsync({
      n: 9,
      field: 1,
      dt: 0.2,
      steps: 40,
      clockA: 4,
      clockB: 0,
      physicalSlices: 15,
      embedDim: 2,
      referenceSite: 0,
    })
      .then(setResult)
      .finally(() => setLoading(false))
  }, [])

  return (
    <Card>
      <CardHeader>
        <CardTitle>Emergent simultaneity surfaces</CardTitle>
        <CardDescription>
          Two physical clocks on the same trajectory assign different time
          labels to the same states. Track how emergent spatial coords of a
          reference site respond to each clock&apos;s foliation — bent surfaces
          mean no global simultaneity hypersurface exists.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <Button onClick={run} disabled={loading} size="sm">
          {loading ? (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          ) : (
            <Play className="mr-2 h-4 w-4" />
          )}
          Measure foliation bend
        </Button>

        {result && (
          <>
            <div className="flex flex-wrap gap-2 text-xs">
              <Badge variant={result.bendDetected ? 'default' : 'outline'}>
                bend detected = {result.bendDetected ? 'yes' : 'no'}
              </Badge>
              <Badge variant="outline">
                mean τ skew = {result.meanTauSkew.toFixed(3)}
              </Badge>
              <Badge variant="outline">
                slope Δ = {result.slopeDelta.toFixed(3)}
              </Badge>
              <Badge variant="outline">
                {result.elapsedMs.toFixed(0)} ms · {result.backend}
              </Badge>
            </div>
            <SkewChart result={result} />
            <p className="text-xs text-muted-foreground">
              Defect clock (site {result.clockA}) vs edge clock (site{' '}
              {result.clockB}). Emergent x-coord of site {result.referenceSite}{' '}
              vs physical time has slope {result.slopeA.toFixed(3)} under A and{' '}
              {result.slopeB.toFixed(3)} under B — different foliations bend
              equal-time surfaces in emergent geometry.
            </p>
          </>
        )}
      </CardContent>
    </Card>
  )
}
