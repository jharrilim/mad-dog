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
import type { HolographyRunResult } from '@/sim/runner'
import { runHolographyAsync, type HolographyRunResultWithBackend } from '@/sim/runner-async'

const GROUND = 'oklch(0.72 0.16 290)'
const RANDOM = 'oklch(0.78 0.13 60)'

function niceMax(v: number): number {
  if (v <= 0) return 1
  const pow = Math.pow(10, Math.floor(Math.log10(v)))
  const n = v / pow
  const step = n <= 1 ? 1 : n <= 2 ? 2 : n <= 5 ? 5 : 10
  return step * pow
}

function AreaLawChart({ result }: { result: HolographyRunResult }) {
  const data = result.report.areaLaw
  const W = 460
  const H = 300
  const pad = { l: 44, r: 16, t: 16, b: 38 }
  const plotW = W - pad.l - pad.r
  const plotH = H - pad.t - pad.b
  const maxSize = data.length + 1
  const maxY = niceMax(
    Math.max(...data.map((p) => Math.max(p.sGround, p.sRandom))),
  )
  const x = (size: number) => pad.l + (size / maxSize) * plotW
  const y = (val: number) => pad.t + plotH - (val / maxY) * plotH

  const line = (key: 'sGround' | 'sRandom') =>
    data.map((p) => `${x(p.size)},${y(p[key])}`).join(' ')

  const yTicks = 4
  const xTicks = data.map((p) => p.size).filter((_, i) => i % 2 === 0)

  return (
    <svg
      viewBox={`0 0 ${W} ${H}`}
      className="w-full h-auto"
      role="img"
      aria-label="Region entropy versus region size: area law vs volume law"
    >
      {/* y gridlines + labels */}
      {Array.from({ length: yTicks + 1 }, (_, i) => {
        const val = (maxY * i) / yTicks
        const yy = y(val)
        return (
          <g key={i}>
            <line
              x1={pad.l}
              y1={yy}
              x2={W - pad.r}
              y2={yy}
              className="stroke-border"
              strokeWidth={0.5}
            />
            <text
              x={pad.l - 6}
              y={yy + 3}
              textAnchor="end"
              className="fill-muted-foreground text-[9px]"
            >
              {val.toFixed(1)}
            </text>
          </g>
        )
      })}
      {/* x labels */}
      {xTicks.map((s) => (
        <text
          key={s}
          x={x(s)}
          y={H - pad.b + 14}
          textAnchor="middle"
          className="fill-muted-foreground text-[9px]"
        >
          {s}
        </text>
      ))}
      <text
        x={pad.l + plotW / 2}
        y={H - 4}
        textAnchor="middle"
        className="fill-muted-foreground text-[10px]"
      >
        region size |A|  (qubits)
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

      {/* volume-law reference (random) */}
      <polyline
        points={line('sRandom')}
        fill="none"
        stroke={RANDOM}
        strokeWidth={2}
        strokeDasharray="5 3"
      />
      {data.map((p) => (
        <circle key={`r${p.size}`} cx={x(p.size)} cy={y(p.sRandom)} r={2.5} fill={RANDOM} />
      ))}

      {/* area-law ground state */}
      <polyline points={line('sGround')} fill="none" stroke={GROUND} strokeWidth={2} />
      {data.map((p) => (
        <circle key={`g${p.size}`} cx={x(p.size)} cy={y(p.sGround)} r={2.5} fill={GROUND} />
      ))}

      {/* legend */}
      <g transform={`translate(${pad.l + 8}, ${pad.t + 6})`}>
        <line x1={0} y1={0} x2={18} y2={0} stroke={GROUND} strokeWidth={2} />
        <text x={24} y={3} className="fill-foreground text-[9px]">
          ground state (area law)
        </text>
        <line x1={0} y1={14} x2={18} y2={14} stroke={RANDOM} strokeWidth={2} strokeDasharray="5 3" />
        <text x={24} y={17} className="fill-foreground text-[9px]">
          random state (volume law)
        </text>
      </g>
    </svg>
  )
}

function RtScatter({ result }: { result: HolographyRunResult }) {
  const { rtPoints, rtSlope } = result.report
  const W = 460
  const H = 300
  const pad = { l: 44, r: 16, t: 16, b: 38 }
  const plotW = W - pad.l - pad.r
  const plotH = H - pad.t - pad.b
  const maxX = niceMax(Math.max(...rtPoints.map((p) => p.cut), 1e-6))
  const maxY = niceMax(Math.max(...rtPoints.map((p) => p.entropy), 1e-6))
  const x = (v: number) => pad.l + (v / maxX) * plotW
  const y = (v: number) => pad.t + plotH - (v / maxY) * plotH

  // Fit line y = slope * x, clipped to the plot box.
  const xEnd = rtSlope > 0 ? Math.min(maxX, maxY / rtSlope) : maxX
  const yEnd = rtSlope * xEnd

  const ticks = 4

  return (
    <svg
      viewBox={`0 0 ${W} ${H}`}
      className="w-full h-auto"
      role="img"
      aria-label="Region entropy versus boundary mutual-information cut"
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
        boundary cut  ½ Σ I(a:b)   (the emergent "area")
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

      {/* RT prediction line */}
      <line
        x1={x(0)}
        y1={y(0)}
        x2={x(xEnd)}
        y2={y(yEnd)}
        stroke={RANDOM}
        strokeWidth={1.5}
        strokeDasharray="5 3"
      />
      {/* data points, sized faintly by region size */}
      {rtPoints.map((p, i) => (
        <circle
          key={i}
          cx={x(p.cut)}
          cy={y(p.entropy)}
          r={2.6}
          fill={GROUND}
          fillOpacity={0.7}
        />
      ))}

      <text x={W - pad.r - 6} y={pad.t + 12} textAnchor="end" className="fill-muted-foreground text-[9px]">
        dashed: S = {rtSlope.toFixed(2)} · cut
      </text>
    </svg>
  )
}

const DEFAULT = { n: 10, field: 1.5, seed: 7 }

export function HolographyViz() {
  const [config, setConfig] = useState(DEFAULT)
  const [result, setResult] = useState<HolographyRunResultWithBackend | null>(null)
  const [running, setRunning] = useState(false)

  const run = useCallback(() => {
    setRunning(true)
    void runHolographyAsync(config)
      .then(setResult)
      .finally(() => setRunning(false))
  }, [config])

  const update = (key: keyof typeof config, value: number) =>
    setConfig((c) => ({ ...c, [key]: value }))

  return (
    <Card>
      <CardHeader>
        <CardTitle>Live Simulator: A Baby Ryu&ndash;Takayanagi Test</CardTitle>
        <CardDescription>
          If our entanglement-built geometry is genuinely holographic, a region&apos;s
          entropy should be set by its <em>boundary</em>, not its bulk. We test
          two predictions on the same chain: (1) the gapped ground state obeys an
          <strong> area law</strong> while a random state obeys a{' '}
          <strong>volume law</strong>, and (2) the discrete Ryu&ndash;Takayanagi
          relation{' '}
          <code className="font-mono text-xs bg-muted px-1 py-0.5 rounded">
            S(A) ≈ ½ Σ I(a:b)
          </code>{' '}
          &mdash; entropy equals the entanglement crossing the boundary.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="space-y-4">
          <div className="grid sm:grid-cols-2 gap-4">
            <label className="text-sm space-y-1 block">
              <span className="text-muted-foreground">Chain length: {config.n}</span>
              <input
                type="range"
                min={6}
                max={12}
                value={config.n}
                onChange={(e) => update('n', Number(e.target.value))}
                className="w-full accent-primary"
              />
            </label>
            <label className="text-sm space-y-1 block">
              <span className="text-muted-foreground">
                Field h: {config.field.toFixed(1)}{' '}
                {config.field >= 1 ? '(gapped)' : '(near-critical)'}
              </span>
              <input
                type="range"
                min={0.6}
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
            {running ? 'Measuring entropies…' : 'Run the test'}
          </Button>
        </div>

        {result && (
          <div className="space-y-6 border-t pt-6">
            <div className="flex flex-wrap items-center gap-3">
              <Badge variant="secondary">{result.report.n}-site chain</Badge>
              <Badge variant="outline">
                {result.backend === 'wasm' ? 'Rust WASM' : 'TypeScript'}
              </Badge>
              <Badge>RT slope ≈ {result.report.rtSlope.toFixed(2)}</Badge>
              <Badge
                variant={result.report.rtR2 > 0.9 ? 'default' : 'outline'}
              >
                R² = {result.report.rtR2.toFixed(3)}
              </Badge>
              <span className="text-xs text-muted-foreground">
                {result.report.rtPoints.length} regions · {result.elapsedMs.toFixed(0)} ms
              </span>
            </div>

            <div className="grid lg:grid-cols-2 gap-6">
              <div>
                <h4 className="text-sm font-medium mb-2">Area law vs. volume law</h4>
                <AreaLawChart result={result} />
                <p className="text-xs text-muted-foreground mt-2">
                  The ground state&apos;s entropy <strong>saturates</strong> &mdash; it
                  tracks the one-site boundary, not the region size. The random
                  state grows ~|A|·ln2 up to half the chain (a Page curve): pure
                  volume law. Entropy-by-boundary is the holographic fingerprint.
                </p>
              </div>
              <div>
                <h4 className="text-sm font-medium mb-2">
                  Entropy = boundary area (Ryu&ndash;Takayanagi)
                </h4>
                <RtScatter result={result} />
                <p className="text-xs text-muted-foreground mt-2">
                  Each dot is one interval: exact entropy S(A) vs. the
                  entanglement crossing its boundary, ½ Σ I(a:b). They fall on a
                  line of slope ≈ {result.report.rtSlope.toFixed(2)} (the essay
                  predicts 1) with R² = {result.report.rtR2.toFixed(3)} &mdash; a
                  discrete minimal-surface law inside our toy universe.
                </p>
              </div>
            </div>

            <p className="text-xs text-muted-foreground leading-relaxed border-t pt-4">
              <strong className="text-foreground">Reading the result honestly:</strong>{' '}
              this does not prove our emergent time is holographic. It shows
              something more modest and real &mdash; the entanglement structure
              that we turned into <em>space</em> already obeys the entropy-equals-area
              relation at the heart of holography. Push the field toward the
              critical point (h → 1) and the area law degrades toward a
              logarithm: the test has teeth, and it can fail.
            </p>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
