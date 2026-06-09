import { useCallback, useEffect, useRef, useState } from 'react'
import { Play, Pause, Loader2, RotateCcw } from 'lucide-react'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { runSpacetime2D, type Spacetime2DConfig } from '@/sim/runner'
import { measureLightCone, type SpacetimeResult } from '@/sim/spacetime'

function signalColor(v: number) {
  return `oklch(0.72 0.16 290 / ${0.12 + v * 0.88})`
}

function GridEmbedding({
  result,
  rows,
  cols,
  selected,
}: {
  result: SpacetimeResult
  rows: number
  cols: number
  selected: number
}) {
  const slice = result.slices[selected]
  // Global bounds across all slices so the frame does not rescale while playing.
  let minX = Infinity
  let maxX = -Infinity
  let minY = Infinity
  let maxY = -Infinity
  let maxSig = 1e-9
  for (const s of result.slices) {
    for (const c of s.coords) {
      minX = Math.min(minX, c[0])
      maxX = Math.max(maxX, c[0])
      minY = Math.min(minY, c[1])
      maxY = Math.max(maxY, c[1])
    }
    for (const v of s.signal) maxSig = Math.max(maxSig, v)
  }
  const span = Math.max(maxX - minX, maxY - minY) || 1
  const cxm = (minX + maxX) / 2
  const cym = (minY + maxY) / 2
  const W = 360
  const H = 360
  const m = 30
  const scale = (Math.min(W, H) - 2 * m) / span
  const sx = (x: number) => W / 2 + (x - cxm) * scale
  const sy = (y: number) => H / 2 - (y - cym) * scale

  const idx = (r: number, c: number) => r * cols + c
  const edges: [number, number][] = []
  for (let r = 0; r < rows; r++) {
    for (let c = 0; c < cols; c++) {
      if (c + 1 < cols) edges.push([idx(r, c), idx(r, c + 1)])
      if (r + 1 < rows) edges.push([idx(r, c), idx(r + 1, c)])
    }
  }

  return (
    <svg
      viewBox={`0 0 ${W} ${H}`}
      className="w-full h-auto max-w-[380px]"
      role="img"
      aria-label="Emergent 2D geometry at the selected clock reading"
    >
      {edges.map(([i, j], e) => (
        <line
          key={e}
          x1={sx(slice.coords[i][0])}
          y1={sy(slice.coords[i][1])}
          x2={sx(slice.coords[j][0])}
          y2={sy(slice.coords[j][1])}
          stroke="oklch(0.5 0.05 280)"
          strokeWidth={1}
          strokeOpacity={0.5}
        />
      ))}
      {slice.coords.map((c, i) => {
        const intensity = slice.signal[i] / maxSig
        return (
          <circle
            key={i}
            cx={sx(c[0])}
            cy={sy(c[1])}
            r={6 + intensity * 8}
            fill={signalColor(intensity)}
            className="stroke-primary"
            strokeWidth={1.2}
          />
        )
      })}
    </svg>
  )
}

const DEFAULT: Spacetime2DConfig = {
  rows: 3,
  cols: 3,
  field: 1,
  dt: 0.25,
  steps: 18,
}

export function Spacetime2DViz() {
  const [config, setConfig] = useState<Spacetime2DConfig>(DEFAULT)
  const [result, setResult] = useState<SpacetimeResult | null>(null)
  const [selected, setSelected] = useState(0)
  const [running, setRunning] = useState(false)
  const [playing, setPlaying] = useState(false)
  const timer = useRef<number | null>(null)

  const run = useCallback(() => {
    setRunning(true)
    setPlaying(false)
    setTimeout(() => {
      try {
        setResult(runSpacetime2D(config))
        setSelected(0)
      } finally {
        setRunning(false)
      }
    }, 30)
  }, [config])

  useEffect(() => {
    if (!playing || !result) return
    timer.current = window.setInterval(() => {
      setSelected((s) => (s + 1) % result.slices.length)
    }, 350)
    return () => {
      if (timer.current) window.clearInterval(timer.current)
    }
  }, [playing, result])

  const update = <K extends keyof Spacetime2DConfig>(
    key: K,
    value: Spacetime2DConfig[K],
  ) => setConfig((c) => ({ ...c, [key]: value }))

  const cone = result ? measureLightCone(result) : null

  return (
    <Card>
      <CardHeader>
        <CardTitle>Live Simulator: A 2D World Evolving in Emergent Time</CardTitle>
        <CardDescription>
          A local Ising grid with a central defect. The emergent 2D geometry is
          reconstructed (via MDS, aligned to remove rotation/reflection gauge
          freedom) at every clock reading, so you can watch a 2D spatial slice
          evolve and a disturbance spread radially across it.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="space-y-4">
          <div className="grid sm:grid-cols-3 gap-4">
            <label className="text-sm space-y-1 block">
              <span className="text-muted-foreground">Rows: {config.rows}</span>
              <input
                type="range"
                min={2}
                max={4}
                value={config.rows}
                onChange={(e) => update('rows', Number(e.target.value))}
                className="w-full accent-primary"
              />
            </label>
            <label className="text-sm space-y-1 block">
              <span className="text-muted-foreground">Cols: {config.cols}</span>
              <input
                type="range"
                min={2}
                max={4}
                value={config.cols}
                onChange={(e) => update('cols', Number(e.target.value))}
                className="w-full accent-primary"
              />
            </label>
            <label className="text-sm space-y-1 block">
              <span className="text-muted-foreground">
                Clock readings: {config.steps}
              </span>
              <input
                type="range"
                min={10}
                max={24}
                value={config.steps}
                onChange={(e) => update('steps', Number(e.target.value))}
                className="w-full accent-primary"
              />
            </label>
          </div>
          <div className="flex items-center gap-3">
            <Button onClick={run} disabled={running}>
              {running ? (
                <Loader2 className="size-4 animate-spin" />
              ) : (
                <Play className="size-4" />
              )}
              {running ? 'Evolving…' : 'Build 2D spacetime'}
            </Button>
            <span className="text-xs text-muted-foreground">
              2^{config.rows * config.cols} = {2 ** (config.rows * config.cols)}{' '}
              dimensional Hilbert space
            </span>
          </div>
        </div>

        {result && cone && (
          <div className="space-y-4 border-t pt-6">
            <div className="flex flex-wrap items-center gap-3">
              <Badge variant="secondary">
                {config.rows}×{config.cols} grid
              </Badge>
              <Badge>v ≈ {cone.velocity.toFixed(2)} sites/time</Badge>
              <span className="text-xs text-muted-foreground">
                {result.elapsedMs.toFixed(0)} ms
              </span>
            </div>

            <div className="flex justify-center">
              <GridEmbedding
                result={result}
                rows={config.rows}
                cols={config.cols}
                selected={selected}
              />
            </div>

            <div className="flex items-center gap-3 max-w-[460px] mx-auto">
              <Button
                variant="outline"
                size="icon"
                onClick={() => setPlaying((p) => !p)}
              >
                {playing ? (
                  <Pause className="size-4" />
                ) : (
                  <Play className="size-4" />
                )}
              </Button>
              <Button
                variant="outline"
                size="icon"
                onClick={() => {
                  setPlaying(false)
                  setSelected(0)
                }}
              >
                <RotateCcw className="size-4" />
              </Button>
              <input
                type="range"
                min={0}
                max={result.slices.length - 1}
                value={selected}
                onChange={(e) => {
                  setPlaying(false)
                  setSelected(Number(e.target.value))
                }}
                className="flex-1 accent-primary"
              />
              <span className="text-xs text-muted-foreground tabular-nums w-24 text-right">
                k={selected} · t={result.slices[selected].t.toFixed(2)}
              </span>
            </div>
            <p className="text-xs text-muted-foreground text-center">
              Nodes are qubits at their entanglement-derived positions; edges
              are grid neighbours. Brightness marks the propagating disturbance.
            </p>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
