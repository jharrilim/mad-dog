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
  runLightConeCompareAsync,
  type LightConeCompareResultWithBackend,
} from '@/sim/runner-async'
import type { LightCone } from '@/sim/types'

const LATTICE = 'oklch(0.72 0.16 290)'
const MI = 'oklch(0.72 0.14 155)'

function signalColor(v: number) {
  return `oklch(0.72 0.16 290 / ${0.06 + v * 0.94})`
}

function ConeDiagram({
  result,
  latticeCone,
  miCone,
}: {
  result: LightConeCompareResultWithBackend['spacetime']
  latticeCone: LightCone
  miCone: LightCone
}) {
  const rows = result.slices.length
  const cols = result.sites
  let max = 1e-9
  for (const s of result.slices)
    for (const v of s.signal) max = Math.max(max, v)

  const cell = 24
  const padL = 40
  const padT = 18
  const W = padL + cols * cell + 8
  const H = padT + rows * cell + 8
  const cx = padL + (latticeCone.center + 0.5) * cell

  function coneLine(cone: LightCone, color: string, dashed: boolean) {
    const slope = cone.velocity * cone.dt * cell
    const yBot = padT + rows * cell
    const xR = cx + slope * rows
    const xL = cx - slope * rows
    return (
      <g key={color + String(dashed)}>
        <line
          x1={cx}
          y1={padT}
          x2={xR}
          y2={yBot}
          stroke={color}
          strokeWidth={2}
          strokeDasharray={dashed ? '5 4' : undefined}
        />
        <line
          x1={cx}
          y1={padT}
          x2={xL}
          y2={yBot}
          stroke={color}
          strokeWidth={2}
          strokeDasharray={dashed ? '5 4' : undefined}
        />
      </g>
    )
  }

  return (
    <svg viewBox={`0 0 ${W} ${H}`} className="w-full h-auto max-w-lg" role="img">
      {result.slices.map((slice, k) =>
        slice.signal.map((v, i) => (
          <rect
            key={`${k}-${i}`}
            x={padL + i * cell}
            y={padT + k * cell}
            width={cell - 1.5}
            height={cell - 1.5}
            rx={2}
            fill={signalColor(v / max)}
          />
        )),
      )}
      {coneLine(latticeCone, LATTICE, false)}
      {coneLine(miCone, MI, true)}
    </svg>
  )
}

export function LightConeCompareViz() {
  const [result, setResult] =
    useState<LightConeCompareResultWithBackend | null>(null)
  const [loading, setLoading] = useState(false)

  const run = useCallback(() => {
    setLoading(true)
    void runLightConeCompareAsync({
      n: 10,
      field: 1.0,
      dt: 0.2,
      steps: 20,
      seed: 42,
    })
      .then(setResult)
      .finally(() => setLoading(false))
  }, [])

  const cmp = result?.comparison

  return (
    <Card>
      <CardHeader>
        <CardTitle>LR velocity: lattice vs MI distance</CardTitle>
        <CardDescription>
          Compare Lieb–Robinson front speed using combinatorial chain distance
          vs gauge-invariant MI distance (no MDS or Procrustes). Agreement
          suggests the emergent metric tracks the lattice; divergence flags
          where emergence breaks down.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <Button onClick={run} disabled={loading} size="sm">
          {loading ? (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          ) : (
            <Play className="mr-2 h-4 w-4" />
          )}
          Run light-cone compare
        </Button>

        {result && cmp && (
          <>
            <div className="flex flex-wrap gap-2 text-xs">
              <Badge variant="outline">
                v_lattice = {cmp.lattice.velocity.toFixed(3)}
              </Badge>
              <Badge variant="outline">
                v_MI (slice 0) = {cmp.mi.velocity.toFixed(3)}
              </Badge>
              <Badge variant="outline">
                v_MI (time avg) = {cmp.miTimeAvg.velocity.toFixed(3)}
              </Badge>
              <Badge
                variant={
                  Math.abs(cmp.velocityRatio - 1) < 0.25
                    ? 'secondary'
                    : 'destructive'
                }
              >
                ratio = {cmp.velocityRatio.toFixed(3)}
              </Badge>
              <Badge variant="outline">
                {result.elapsedMs.toFixed(0)} ms · {result.backend}
              </Badge>
            </div>

            <ConeDiagram
              result={result.spacetime}
              latticeCone={cmp.lattice}
              miCone={cmp.mi}
            />
            <p className="text-xs text-muted-foreground">
              Solid lines: lattice distance fit. Dashed: MI distance from
              slice-0 state (fixed emergent metric).
            </p>
          </>
        )}
      </CardContent>
    </Card>
  )
}
