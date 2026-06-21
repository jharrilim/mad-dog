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
import { runRelationalTimeAsync, type DualClockResultWithBackend } from '@/sim/runner-async'
import type { DualClockResult } from '@/sim/relational-time'

function signalColor(v: number) {
  return `oklch(0.72 0.16 290 / ${0.06 + v * 0.94})`
}

function MiniSpacetime({
  result,
  title,
}: {
  result: DualClockResult['uniform']['result']
  title: string
}) {
  const rows = result.slices.length
  const cols = result.sites
  let max = 1e-9
  for (const s of result.slices)
    for (const v of s.signal) max = Math.max(max, v)

  const cell = 22
  const padL = 36
  const padT = 14
  const W = padL + cols * cell + 4
  const H = padT + rows * cell + 4

  return (
    <div>
      <h4 className="text-sm font-medium mb-2">{title}</h4>
      <svg
        viewBox={`0 0 ${W} ${H}`}
        className="w-full h-auto max-w-[380px]"
        role="img"
        aria-label={title}
      >
        <text x={padL} y={10} className="fill-muted-foreground text-[9px]">
          space →
        </text>
        <text
          x={8}
          y={padT + 6}
          className="fill-muted-foreground text-[9px]"
          transform={`rotate(-90 8 ${padT + 6})`}
        >
          time ↓
        </text>
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
      </svg>
    </div>
  )
}

function TimeMapChart({ result }: { result: DualClockResult }) {
  const { timeMap } = result
  const W = 460
  const H = 280
  const pad = { l: 44, r: 16, t: 16, b: 38 }
  const plotW = W - pad.l - pad.r
  const plotH = H - pad.t - pad.b
  const maxX = Math.max(...timeMap.map((p) => p.tauPhysical), 1)
  const maxY = Math.max(...timeMap.map((p) => p.tauUniform), 1)
  const x = (v: number) => pad.l + (v / maxX) * plotW
  const y = (v: number) => pad.t + plotH - (v / maxY) * plotH

  const { syncSlope, syncIntercept } = result
  const xEnd = maxX
  const yEnd = syncSlope * xEnd + syncIntercept

  return (
    <svg
      viewBox={`0 0 ${W} ${H}`}
      className="w-full h-auto"
      role="img"
      aria-label="Relational time map between two clocks"
    >
      {[0, 0.25, 0.5, 0.75, 1].map((f) => {
        const yy = pad.t + plotH * (1 - f)
        const xx = pad.l + plotW * f
        return (
          <g key={f}>
            <line x1={pad.l} y1={yy} x2={W - pad.r} y2={yy} className="stroke-border" strokeWidth={0.5} />
            <text x={pad.l - 6} y={yy + 3} textAnchor="end" className="fill-muted-foreground text-[9px]">
              {(maxY * f).toFixed(0)}
            </text>
            <text x={xx} y={H - pad.b + 14} textAnchor="middle" className="fill-muted-foreground text-[9px]">
              {(maxX * f).toFixed(0)}
            </text>
          </g>
        )
      })}
      <text x={pad.l + plotW / 2} y={H - 4} textAnchor="middle" className="fill-muted-foreground text-[10px]">
        physical clock reading
      </text>
      <text
        x={12}
        y={pad.t + plotH / 2}
        textAnchor="middle"
        transform={`rotate(-90 12 ${pad.t + plotH / 2})`}
        className="fill-muted-foreground text-[10px]"
      >
        uniform clock reading (at same state)
      </text>

      {/* perfect-sync reference diagonal */}
      <line
        x1={x(0)}
        y1={y(0)}
        x2={x(maxX)}
        y2={y(maxX * (maxY / maxX))}
        className="stroke-muted-foreground/40"
        strokeWidth={1}
        strokeDasharray="4 3"
      />

      {/* fitted line */}
      <line
        x1={x(0)}
        y1={y(Math.max(0, syncIntercept))}
        x2={x(xEnd)}
        y2={y(Math.min(maxY, yEnd))}
        stroke="oklch(0.78 0.13 60)"
        strokeWidth={1.5}
        strokeDasharray="5 3"
      />

      {timeMap.map((p, i) => (
        <circle key={i} cx={x(p.tauPhysical)} cy={y(p.tauUniform)} r={4} fill="oklch(0.72 0.16 290)" />
      ))}
    </svg>
  )
}

const DEFAULT = {
  n: 9,
  field: 1,
  dt: 0.2,
  steps: 40,
  clockSite: 0,
  physicalSlices: 15,
}

export function RelationalTimeViz() {
  const [config, setConfig] = useState(DEFAULT)
  const [result, setResult] = useState<DualClockResultWithBackend | null>(null)
  const [running, setRunning] = useState(false)

  const run = useCallback(() => {
    setRunning(true)
    void runRelationalTimeAsync(config)
      .then(setResult)
      .finally(() => setRunning(false))
  }, [config])

  const update = (key: keyof typeof config, value: number) =>
    setConfig((c) => ({ ...c, [key]: value }))

  const center = Math.floor(config.n / 2)

  return (
    <Card>
      <CardHeader>
        <CardTitle>Live Simulator: Two Clocks, Two Times</CardTitle>
        <CardDescription>
          Page&ndash;Wootters time is relational: it depends on which clock you
          use. We evolve <em>one</em> quenched chain and read it with two clocks
          &mdash; a uniform Δt clock and a physical clock that ticks when the
          disturbance reaches a chosen site. The same history, two different
          emergent time coordinates. Neither is &ldquo;the&rdquo; time.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="space-y-4">
          <div className="grid sm:grid-cols-3 gap-4">
            <label className="text-sm space-y-1 block">
              <span className="text-muted-foreground">Chain length: {config.n}</span>
              <input
                type="range"
                min={7}
                max={11}
                value={config.n}
                onChange={(e) => update('n', Number(e.target.value))}
                className="w-full accent-primary"
              />
            </label>
            <label className="text-sm space-y-1 block">
              <span className="text-muted-foreground">
                Physical clock site: {config.clockSite}
                {config.clockSite === center ? ' (defect)' : config.clockSite === 0 ? ' (edge)' : ''}
              </span>
              <input
                type="range"
                min={0}
                max={config.n - 1}
                value={config.clockSite}
                onChange={(e) => update('clockSite', Number(e.target.value))}
                className="w-full accent-primary"
              />
            </label>
            <label className="text-sm space-y-1 block">
              <span className="text-muted-foreground">Trajectory steps: {config.steps}</span>
              <input
                type="range"
                min={24}
                max={50}
                value={config.steps}
                onChange={(e) => update('steps', Number(e.target.value))}
                className="w-full accent-primary"
              />
            </label>
          </div>
          <Button onClick={run} disabled={running}>
            {running ? (
              <Loader2 className="size-4 animate-spin" />
            ) : (
              <Play className="size-4" />
            )}
            {running ? 'Comparing clocks…' : 'Compare clocks'}
          </Button>
        </div>

        {result && (
          <div className="space-y-6 border-t pt-6">
            <div className="flex flex-wrap items-center gap-3">
              <Badge variant="secondary">{result.sites}-site chain</Badge>
              <Badge variant={result.syncR2 > 0.95 ? 'outline' : 'default'}>
                sync R² = {result.syncR2.toFixed(3)}
              </Badge>
              <Badge variant="outline">
                rate ratio ≈ {result.syncSlope.toFixed(2)}
              </Badge>
              <span className="text-xs text-muted-foreground">
                {result.elapsedMs.toFixed(0)} ms · {result.backend}
              </span>
            </div>

            <div className="grid lg:grid-cols-2 gap-6">
              <div>
                <h4 className="text-sm font-medium mb-2">Relational time map</h4>
                <TimeMapChart result={result} />
                <p className="text-xs text-muted-foreground mt-2">
                  Each dot: at physical-clock tick τ_B, the uniform clock was at
                  reading τ_A for the <em>same</em> quantum state. A straight
                  diagonal means the clocks agree. A kink or plateau means one
                  clock stalled while the other kept ticking &mdash; time is
                  relational, not absolute. Try moving the physical clock to the
                  defect site ({center}) vs an edge (0).
                </p>
              </div>
              <div className="space-y-4">
                <MiniSpacetime
                  result={result.uniform.result}
                  title="History A: uniform clock (fixed Δt)"
                />
                <MiniSpacetime
                  result={result.physical.result}
                  title={`History B: physical clock (site ${config.clockSite})`}
                />
                <p className="text-xs text-muted-foreground">
                  Same underlying quench, different time labels on the vertical
                  axis. History B&apos;s rows are physical-clock ticks, not the
                  same Δt as History A &mdash; so the light cones look stretched
                  or compressed relative to each other.
                </p>
              </div>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
