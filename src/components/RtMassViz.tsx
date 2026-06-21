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
import type { RtMassRunResult } from '@/sim/runner'
import { runRtMassAsync, type RtMassRunResultWithBackend } from '@/sim/runner-async'

const VACUUM = 'oklch(0.55 0.02 290 / 0.55)'

function massColor(dist: number, maxDist: number) {
  // Near mass: warm (high hue), far: cool primary.
  const t = maxDist > 0 ? dist / maxDist : 0
  const hue = 290 - t * 80
  return `oklch(0.72 0.16 ${hue})`
}

function niceMax(v: number): number {
  if (v <= 0) return 1
  const pow = Math.pow(10, Math.floor(Math.log10(v)))
  const n = v / pow
  const step = n <= 1 ? 1 : n <= 2 ? 2 : n <= 5 ? 5 : 10
  return step * pow
}

function DualRtScatter({ result }: { result: RtMassRunResult }) {
  const { vacuum, mass } = result.report
  const allPts = [...vacuum.rtPoints, ...mass.rtPoints]
  const maxX = niceMax(Math.max(...allPts.map((p) => p.cut), 1e-6))
  const maxY = niceMax(Math.max(...allPts.map((p) => p.entropy), 1e-6))
  const maxDist = Math.max(
    ...mass.rtPoints.map((p) => p.distFromMass ?? 0),
    1,
  )

  const W = 460
  const H = 300
  const pad = { l: 44, r: 16, t: 16, b: 38 }
  const plotW = W - pad.l - pad.r
  const plotH = H - pad.t - pad.b
  const x = (v: number) => pad.l + (v / maxX) * plotW
  const y = (v: number) => pad.t + plotH - (v / maxY) * plotH
  const ticks = 4

  const lineEnd = (slope: number) => {
    const xEnd = slope > 0 ? Math.min(maxX, maxY / slope) : maxX
    return { x1: x(0), y1: y(0), x2: x(xEnd), y2: y(slope * xEnd) }
  }
  const vacLine = lineEnd(vacuum.rtSlope)
  const massLine = lineEnd(mass.rtSlope)

  return (
    <svg
      viewBox={`0 0 ${W} ${H}`}
      className="w-full h-auto"
      role="img"
      aria-label="RT relation vacuum vs mass-deformed"
    >
      {Array.from({ length: ticks + 1 }, (_, i) => {
        const yy = pad.t + (plotH * i) / ticks
        const xx = pad.l + (plotW * i) / ticks
        return (
          <g key={i}>
            <line x1={pad.l} y1={yy} x2={W - pad.r} y2={yy} className="stroke-border" strokeWidth={0.5} />
            <text x={pad.l - 6} y={yy + 3} textAnchor="end" className="fill-muted-foreground text-[9px]">
              {(maxY * (ticks - i) / ticks).toFixed(2)}
            </text>
            <text x={xx} y={H - pad.b + 14} textAnchor="middle" className="fill-muted-foreground text-[9px]">
              {(maxX * i / ticks).toFixed(2)}
            </text>
          </g>
        )
      })}
      <text x={pad.l + plotW / 2} y={H - 4} textAnchor="middle" className="fill-muted-foreground text-[10px]">
        boundary cut  ½ Σ I(a:b)
      </text>
      <text
        x={12}
        y={pad.t + plotH / 2}
        textAnchor="middle"
        transform={`rotate(-90 12 ${pad.t + plotH / 2})`}
        className="fill-muted-foreground text-[10px]"
      >
        entropy S(A)
      </text>

      <line {...vacLine} stroke={VACUUM} strokeWidth={2} strokeDasharray="5 3" />
      <line
        {...massLine}
        stroke="oklch(0.78 0.13 60)"
        strokeWidth={1.5}
        strokeDasharray="4 3"
      />

      {vacuum.rtPoints.map((p, i) => (
        <circle key={`v${i}`} cx={x(p.cut)} cy={y(p.entropy)} r={2} fill={VACUUM} />
      ))}
      {mass.rtPoints.map((p, i) => (
        <circle
          key={`m${i}`}
          cx={x(p.cut)}
          cy={y(p.entropy)}
          r={3.2}
          fill={massColor(p.distFromMass ?? 0, maxDist)}
          fillOpacity={0.85}
        />
      ))}

      <g transform={`translate(${pad.l + 8}, ${pad.t + 6})`}>
        <line x1={0} y1={0} x2={16} y2={0} stroke={VACUUM} strokeWidth={2} strokeDasharray="5 3" />
        <text x={22} y={3} className="fill-foreground text-[9px]">
          vacuum (slope {vacuum.rtSlope.toFixed(2)})
        </text>
        <line x1={0} y1={14} x2={16} y2={14} stroke="oklch(0.78 0.13 60)" strokeWidth={1.5} strokeDasharray="4 3" />
        <text x={22} y={17} className="fill-foreground text-[9px]">
          with mass (slope {mass.rtSlope.toFixed(2)})
        </text>
      </g>
    </svg>
  )
}

function SlopeSweepChart({ result }: { result: RtMassRunResult }) {
  const { sweep, strength } = result.report
  const W = 460
  const H = 220
  const pad = { l: 44, r: 16, t: 16, b: 38 }
  const plotW = W - pad.l - pad.r
  const plotH = H - pad.t - pad.b
  const maxX = sweep[sweep.length - 1]?.strength ?? 2
  const minY = Math.min(...sweep.map((p) => p.slope), 0.8)
  const maxY = Math.max(...sweep.map((p) => p.slope), 1.2)
  const yPad = (maxY - minY) * 0.15 || 0.1
  const yLo = minY - yPad
  const yHi = maxY + yPad

  const x = (s: number) => pad.l + (s / maxX) * plotW
  const y = (v: number) => pad.t + plotH - ((v - yLo) / (yHi - yLo)) * plotH
  const pts = sweep.map((p) => `${x(p.strength)},${y(p.slope)}`).join(' ')

  return (
    <svg
      viewBox={`0 0 ${W} ${H}`}
      className="w-full h-auto"
      role="img"
      aria-label="RT slope versus mass strength"
    >
      <line
        x1={pad.l}
        y1={y(1)}
        x2={W - pad.r}
        y2={y(1)}
        className="stroke-muted-foreground/50"
        strokeWidth={1}
        strokeDasharray="4 3"
      />
      <text x={W - pad.r} y={y(1) - 4} textAnchor="end" className="fill-muted-foreground text-[9px]">
        slope = 1 (vacuum prediction)
      </text>
      <polyline points={pts} fill="none" stroke="oklch(0.72 0.16 290)" strokeWidth={2} />
      {sweep.map((p) => (
        <circle key={p.strength} cx={x(p.strength)} cy={y(p.slope)} r={3} fill="oklch(0.72 0.16 290)" />
      ))}
      {/* current strength marker */}
      <line
        x1={x(strength)}
        y1={pad.t}
        x2={x(strength)}
        y2={H - pad.b}
        stroke="oklch(0.78 0.13 60)"
        strokeWidth={1.5}
        strokeDasharray="3 2"
      />
      <text x={pad.l + plotW / 2} y={H - 4} textAnchor="middle" className="fill-muted-foreground text-[10px]">
        mass strength (quench time)
      </text>
      <text
        x={12}
        y={pad.t + plotH / 2}
        textAnchor="middle"
        transform={`rotate(-90 12 ${pad.t + plotH / 2})`}
        className="fill-muted-foreground text-[10px]"
      >
        RT slope
      </text>
    </svg>
  )
}

const DEFAULT = { n: 10, field: 1.5, seed: 7, strength: 1.0 }

export function RtMassViz() {
  const [config, setConfig] = useState(DEFAULT)
  const [result, setResult] = useState<RtMassRunResultWithBackend | null>(null)
  const [running, setRunning] = useState(false)

  const run = useCallback(() => {
    setRunning(true)
    void runRtMassAsync(config)
      .then(setResult)
      .finally(() => setRunning(false))
  }, [config])

  const update = (key: keyof typeof config, value: number) =>
    setConfig((c) => ({ ...c, [key]: value }))

  const delta =
    result
      ? result.report.mass.rtSlope - result.report.vacuum.rtSlope
      : 0

  return (
    <Card>
      <CardHeader>
        <CardTitle>Live Simulator: Mass Curves the RT Slope</CardTitle>
        <CardDescription>
          In holography, energy curves the emergent geometry. Here &ldquo;mass&rdquo;
          is a local spin flip at the chain centre, evolved briefly so
          entanglement concentrates around it. We ask: does the discrete
          Ryu&ndash;Takayanagi slope{' '}
          <code className="font-mono text-xs bg-muted px-1 py-0.5 rounded">
            S(A) ≈ slope · ½ Σ I(a:b)
          </code>{' '}
          deform away from 1, as an emergent metric might under stress?
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="space-y-4">
          <div className="grid sm:grid-cols-2 gap-4">
            <label className="text-sm space-y-1 block">
              <span className="text-muted-foreground">
                Mass strength: {config.strength.toFixed(2)}{' '}
                {config.strength === 0 ? '(vacuum)' : '(local excitation)'}
              </span>
              <input
                type="range"
                min={0}
                max={2}
                step={0.05}
                value={config.strength}
                onChange={(e) => update('strength', Number(e.target.value))}
                className="w-full accent-primary"
              />
            </label>
            <label className="text-sm space-y-1 block">
              <span className="text-muted-foreground">
                Field h: {config.field.toFixed(1)} (gapped vacuum)
              </span>
              <input
                type="range"
                min={1}
                max={2.5}
                step={0.1}
                value={config.field}
                onChange={(e) => update('field', Number(e.target.value))}
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
            {running ? 'Measuring deformation…' : 'Inject mass & measure'}
          </Button>
        </div>

        {result && (
          <div className="space-y-6 border-t pt-6">
            <div className="flex flex-wrap items-center gap-3">
              <Badge variant="secondary">
                mass at site {result.report.massSite}
              </Badge>
              <Badge variant="outline">
                {result.backend === 'wasm' ? 'Rust WASM' : 'TypeScript'}
              </Badge>
              <Badge>
                vacuum slope {result.report.vacuum.rtSlope.toFixed(2)}
              </Badge>
              <Badge variant={Math.abs(delta) > 0.05 ? 'default' : 'outline'}>
                mass slope {result.report.mass.rtSlope.toFixed(2)}{' '}
                ({delta >= 0 ? '+' : ''}{delta.toFixed(2)})
              </Badge>
              <Badge variant="outline">
                R² {result.report.mass.rtR2.toFixed(3)}
              </Badge>
              <span className="text-xs text-muted-foreground">
                {result.elapsedMs.toFixed(0)} ms
              </span>
            </div>

            <div className="grid lg:grid-cols-2 gap-6">
              <div>
                <h4 className="text-sm font-medium mb-2">
                  RT relation: vacuum vs. mass
                </h4>
                <DualRtScatter result={result} />
                <p className="text-xs text-muted-foreground mt-2">
                  Gray = gapped ground state (slope ≈ 1). Coloured dots = state
                  with local excitation; colour is distance from the mass site
                  (warm = near mass). The fit line <strong>tilts upward</strong>
                  : concentrated entanglement makes entropy exceed the
                  boundary-area prediction — a discrete hint of curvature.
                </p>
              </div>
              <div>
                <h4 className="text-sm font-medium mb-2">
                  Slope vs. mass strength
                </h4>
                <SlopeSweepChart result={result} />
                <p className="text-xs text-muted-foreground mt-2">
                  The dashed horizontal is the vacuum prediction (slope 1).
                  As mass grows, the slope rises and R² dips — the linear
                  RT law is strained but not destroyed. Set mass to 0 to recover
                  the vacuum; push it hard and watch the deformation grow.
                </p>
              </div>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
