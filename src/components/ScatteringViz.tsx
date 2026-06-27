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
import {
  timeSeriesChartPath,
  worldlinePolylinePoints,
  WORLDLINE_COLORS,
} from '@/components/worldline-viz'

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
  const layout = { cell, padL, padT }

  const [wl0, wl1] = result.worldlines

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
      <polyline
        fill="none"
        stroke={WORLDLINE_COLORS[0]}
        strokeWidth={2.5}
        points={worldlinePolylinePoints(wl0, layout)}
      />
      <polyline
        fill="none"
        stroke={WORLDLINE_COLORS[1]}
        strokeWidth={2.5}
        points={worldlinePolylinePoints(wl1, layout)}
      />
    </svg>
  )
}

function SeparationChart({
  series,
  overlapStep,
}: {
  series: number[]
  overlapStep?: number
}) {
  const W = 280
  const H = 72
  const { line, maxY, markerX } = timeSeriesChartPath(series, W, H, {
    markerStep: overlapStep,
  })
  if (!line) return null

  return (
    <div className="space-y-1">
      <h4 className="text-sm font-medium">Separation vs emergent time</h4>
      <svg viewBox={`0 0 ${W} ${H}`} className="w-full max-w-sm h-auto" role="img">
        <line x1={8} y1={H - 8} x2={W - 8} y2={H - 8} className="stroke-border" />
        <line x1={8} y1={8} x2={8} y2={H - 8} className="stroke-border" />
        {markerX !== undefined && (
          <line
            x1={markerX}
            y1={8}
            x2={markerX}
            y2={H - 8}
            className="stroke-muted-foreground"
            strokeDasharray="3 2"
            strokeWidth={1}
          />
        )}
        <polyline fill="none" stroke={WORLDLINE_COLORS[0]} strokeWidth={2} points={line} />
        <text x={12} y={14} className="fill-muted-foreground text-[9px]">
          {maxY.toFixed(0)} sites
        </text>
        <text x={W - 8} y={H - 2} textAnchor="end" className="fill-muted-foreground text-[9px]">
          time →
        </text>
      </svg>
      <p className="text-xs text-muted-foreground">
        Lattice separation between tracked excitation centroids. A dip toward zero
        marks cone overlap; dashed line = overlap step.
      </p>
    </div>
  )
}

function PhaseChart({
  series,
  overlapStep,
}: {
  series: number[]
  overlapStep?: number
}) {
  const W = 280
  const H = 72
  const { line, minY, maxY, markerX } = timeSeriesChartPath(series, W, H, {
    markerStep: overlapStep,
  })
  if (!line) return null

  return (
    <div className="space-y-1">
      <h4 className="text-sm font-medium">Exchange phase vs emergent time</h4>
      <svg viewBox={`0 0 ${W} ${H}`} className="w-full max-w-sm h-auto" role="img">
        <line x1={8} y1={H - 8} x2={W - 8} y2={H - 8} className="stroke-border" />
        <line x1={8} y1={8} x2={8} y2={H - 8} className="stroke-border" />
        {markerX !== undefined && (
          <line
            x1={markerX}
            y1={8}
            x2={markerX}
            y2={H - 8}
            className="stroke-muted-foreground"
            strokeDasharray="3 2"
            strokeWidth={1}
          />
        )}
        <polyline fill="none" stroke={WORLDLINE_COLORS[1]} strokeWidth={2} points={line} />
        <text x={12} y={14} className="fill-muted-foreground text-[9px]">
          {maxY.toFixed(2)} rad
        </text>
        <text x={12} y={H - 12} className="fill-muted-foreground text-[9px]">
          {minY.toFixed(2)}
        </text>
        <text x={W - 8} y={H - 2} textAnchor="end" className="fill-muted-foreground text-[9px]">
          time →
        </text>
      </svg>
      <p className="text-xs text-muted-foreground">
        Two-defect exchange phase δ = arg(ψ₁₁) + arg(ψ₀₀) − arg(ψ₁₀) − arg(ψ₀₁). Kink at
        overlap marks interaction phase shift.
      </p>
    </div>
  )
}

export function ScatteringViz() {
  const [field, setField] = useState(0.7)
  const [result, setResult] = useState<ScatteringResultWithBackend | null>(null)
  const [loading, setLoading] = useState(false)

  const run = useCallback(() => {
    setLoading(true)
    void runScatteringAsync({
      n: 12,
      field,
      dt: 0.12,
      steps: 40,
      defectSites: [3, 8],
      lite: false,
    })
      .then(setResult)
      .finally(() => setLoading(false))
  }, [field])

  return (
    <Card>
      <CardHeader>
        <CardTitle>Two-defect scattering</CardTitle>
        <CardDescription>
          Two Z-defects in ordered-phase TFIM. Tracks whether both excitations
          propagate (worldlines move) without binding (min separation ≥ 1). Lower
          h sharpens worldlines; higher h smears the signal.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="flex flex-wrap items-center gap-4">
          <label className="text-sm flex items-center gap-2">
            <span className="text-muted-foreground whitespace-nowrap">
              Field h = {field.toFixed(1)}
            </span>
            <input
              type="range"
              min={0.5}
              max={1.2}
              step={0.1}
              value={field}
              onChange={(e) => setField(Number(e.target.value))}
              className="w-32 accent-primary"
            />
          </label>
          <Button onClick={run} disabled={loading} size="sm">
            {loading ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : (
              <Play className="mr-2 h-4 w-4" />
            )}
            Run scattering
          </Button>
        </div>

        {result && (
          <>
            <div className="flex flex-wrap gap-2 text-xs">
              <Badge variant={result.bothMoved ? 'default' : 'destructive'}>
                both moved: {result.bothMoved ? 'yes' : 'no'}
              </Badge>
              <Badge variant={result.crossed ? 'secondary' : 'outline'}>
                crossed: {result.crossed ? 'yes' : 'no'}
              </Badge>
              <Badge variant="outline">
                min separation = {result.minSeparation.toFixed(1)} sites
              </Badge>
              <Badge variant={result.overlapDetected ? 'default' : 'outline'}>
                overlap step = {result.overlapStep}
              </Badge>
              <Badge variant="outline">
                phase shift = {result.interactionPhaseShift.toFixed(3)} rad
              </Badge>
              <Badge variant="outline">
                time delay = {result.separationTimeDelay.toFixed(3)}
              </Badge>
              <Badge variant={result.phaseStable ? 'default' : 'secondary'}>
                phase stable: {result.phaseStable ? 'yes' : 'no'}
              </Badge>
              <Badge variant="outline">
                v₀ ≈ {result.velocities[0].toFixed(2)} · v₁ ≈{' '}
                {result.velocities[1].toFixed(2)} sites/time
              </Badge>
              <Badge variant="outline">
                defects at {result.defectSites.join(', ')}
              </Badge>
              <Badge variant="outline">
                {result.elapsedMs.toFixed(0)} ms · {result.backend}
              </Badge>
            </div>
            <div className="grid lg:grid-cols-2 gap-6 items-start">
              <ScatteringDiagram result={result} />
              <div className="space-y-6">
                <SeparationChart
                  series={result.separationSeries}
                  overlapStep={result.overlapStep}
                />
                <PhaseChart series={result.phaseSeries} overlapStep={result.overlapStep} />
              </div>
            </div>
          </>
        )}
      </CardContent>
    </Card>
  )
}
