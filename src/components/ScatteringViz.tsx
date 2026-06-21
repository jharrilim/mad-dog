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
  runScatteringAsync,
  type ScatteringResultWithBackend,
} from '@/sim/runner-async'

const WL0 = 'oklch(0.72 0.16 290)'
const WL1 = 'oklch(0.72 0.14 155)'

function ScatteringDiagram({ result }: { result: ScatteringResultWithBackend }) {
  const rows = result.slices.length
  const cols = result.n
  let max = 1e-9
  for (const s of result.slices)
    for (const v of s.signal) max = Math.max(max, v)

  const cell = 22
  const padL = 36
  const padT = 14
  const W = padL + cols * cell + 4
  const H = padT + rows * cell + 4

  const [wl0, wl1] = result.worldlines
  const toXY = (wl: typeof wl0) =>
    wl.map((p, k) => {
      const x = padL + p.site * cell + cell / 2
      const y = padT + k * cell + cell / 2
      return `${x},${y}`
    }).join(' ')

  return (
    <svg viewBox={`0 0 ${W} ${H}`} className="w-full h-auto max-w-xl" role="img">
      {result.slices.map((slice, k) =>
        slice.signal.map((v, i) => (
          <rect
            key={`${k}-${i}`}
            x={padL + i * cell}
            y={padT + k * cell}
            width={cell - 1.5}
            height={cell - 1.5}
            rx={2}
            fill={`oklch(0.72 0.16 290 / ${0.06 + (v / max) * 0.94})`}
          />
        )),
      )}
      <polyline fill="none" stroke={WL0} strokeWidth={2.5} points={toXY(wl0)} />
      <polyline fill="none" stroke={WL1} strokeWidth={2.5} points={toXY(wl1)} />
    </svg>
  )
}

export function ScatteringViz() {
  const [result, setResult] = useState<ScatteringResultWithBackend | null>(null)
  const [loading, setLoading] = useState(false)

  const run = useCallback(() => {
    setLoading(true)
    void runScatteringAsync({
      n: 12,
      field: 0.7,
      dt: 0.12,
      steps: 40,
      defectSites: [3, 8],
      lite: false,
    })
      .then(setResult)
      .finally(() => setLoading(false))
  }, [])

  return (
    <Card>
      <CardHeader>
        <CardTitle>Two-defect scattering</CardTitle>
        <CardDescription>
          Two Z-defects in ordered-phase TFIM (h ≈ 0.7). Tracks whether both
          excitations propagate (worldlines move) without binding (min separation
          ≥ 1). Full JW pass-through crossing is not resolved in this signal
          picture — see crossed badge.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <Button onClick={run} disabled={loading} size="sm">
          {loading ? (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          ) : (
            <Play className="mr-2 h-4 w-4" />
          )}
          Run scattering
        </Button>

        {result && (
          <>
            <div className="flex flex-wrap gap-2 text-xs">
              <Badge variant={result.crossed ? 'secondary' : 'destructive'}>
                crossed: {result.crossed ? 'yes' : 'no'}
              </Badge>
              <Badge variant="outline">
                min separation = {result.minSeparation.toFixed(1)} sites
              </Badge>
              <Badge variant="outline">
                defects at {result.defectSites.join(', ')}
              </Badge>
              <Badge variant="outline">
                {result.elapsedMs.toFixed(0)} ms · {result.backend}
              </Badge>
            </div>
            <ScatteringDiagram result={result} />
          </>
        )}
      </CardContent>
    </Card>
  )
}
