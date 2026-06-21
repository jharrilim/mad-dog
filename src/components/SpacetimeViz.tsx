import { useCallback, useMemo, useState } from 'react'
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
  type SpacetimeRunConfig,
} from '@/sim/runner'
import { runSpacetimeAsync, type SpacetimeResultWithBackend } from '@/sim/runner-async'
import {
  measureLightCone,
  type LightCone,
  type SpacetimeResult,
} from '@/sim/spacetime'

function signalColor(v: number) {
  // v in [0,1] -> from transparent to bright primary.
  return `oklch(0.72 0.16 290 / ${0.06 + v * 0.94})`
}

function SpacetimeDiagram({
  result,
  cone,
  selected,
  onSelect,
}: {
  result: SpacetimeResult
  cone: LightCone
  selected: number
  onSelect: (k: number) => void
}) {
  const rows = result.slices.length
  const cols = result.sites
  let max = 1e-9
  for (const s of result.slices)
    for (const v of s.signal) max = Math.max(max, v)

  const cell = 26
  const padL = 40
  const padT = 18
  const W = padL + cols * cell + 8
  const H = padT + rows * cell + 8

  // Fitted light-cone lines: site offset = velocity * time = velocity * dt * row.
  const cx = padL + (cone.center + 0.5) * cell
  const slope = cone.velocity * cone.dt * cell // px per row
  const yTop = padT
  const yBot = padT + rows * cell
  const xRight = cx + slope * rows
  const xLeft = cx - slope * rows

  return (
    <svg
      viewBox={`0 0 ${W} ${H}`}
      className="w-full h-auto max-w-[460px]"
      role="img"
      aria-label="Emergent spacetime light-cone diagram"
    >
      <text x={padL} y={12} className="fill-muted-foreground text-[10px]">
        space (qubit) →
      </text>
      <text
        x={10}
        y={padT + 8}
        className="fill-muted-foreground text-[10px]"
        transform={`rotate(-90 10 ${padT + 8})`}
      >
        emergent time ↓
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
            stroke={k === selected ? 'oklch(0.85 0.12 200)' : 'none'}
            strokeWidth={k === selected ? 1.5 : 0}
            onClick={() => onSelect(k)}
            style={{ cursor: 'pointer' }}
          />
        )),
      )}
      {cone.velocity > 0 && (
        <g
          stroke="oklch(0.85 0.14 90)"
          strokeWidth={1.5}
          strokeDasharray="4 3"
          fill="none"
        >
          <line x1={cx} y1={yTop} x2={xRight} y2={yBot} />
          <line x1={cx} y1={yTop} x2={xLeft} y2={yBot} />
        </g>
      )}
    </svg>
  )
}

function SpatialSlice({ result, selected }: { result: SpacetimeResult; selected: number }) {
  const slice = result.slices[selected]
  const coords = slice.coords.map((c) => c[0])
  const minX = Math.min(...coords)
  const maxX = Math.max(...coords)
  const span = maxX - minX || 1
  let max = 1e-9
  for (const v of slice.signal) max = Math.max(max, v)
  const W = 460
  const H = 90
  const m = 24
  const sx = (x: number) => m + ((x - minX) / span) * (W - 2 * m)

  return (
    <svg
      viewBox={`0 0 ${W} ${H}`}
      className="w-full h-auto max-w-[460px]"
      role="img"
      aria-label="Emergent spatial slice at the selected clock reading"
    >
      <line x1={m} y1={H / 2} x2={W - m} y2={H / 2} className="stroke-border" />
      {coords.map((x, i) => {
        const intensity = max > 0 ? slice.signal[i] / max : 0
        return (
          <g key={i}>
            <circle
              cx={sx(x)}
              cy={H / 2}
              r={7 + intensity * 6}
              fill={signalColor(intensity)}
              className="stroke-primary"
              strokeWidth={1.2}
            />
            <text
              x={sx(x)}
              y={H / 2 + 3}
              textAnchor="middle"
              className="fill-foreground text-[8px] font-mono"
            >
              {i}
            </text>
          </g>
        )
      })}
    </svg>
  )
}

const DEFAULT: SpacetimeRunConfig = {
  n: 9,
  field: 1,
  dt: 0.2,
  steps: 20,
  seed: 7,
}

export function SpacetimeViz() {
  const [config, setConfig] = useState<SpacetimeRunConfig>(DEFAULT)
  const [result, setResult] = useState<SpacetimeResultWithBackend | null>(null)
  const [selected, setSelected] = useState(0)
  const [running, setRunning] = useState(false)

  const run = useCallback(() => {
    setRunning(true)
    void runSpacetimeAsync(config)
      .then((r) => {
        setResult(r)
        setSelected(0)
      })
      .finally(() => setRunning(false))
  }, [config])

  const update = <K extends keyof SpacetimeRunConfig>(
    key: K,
    value: SpacetimeRunConfig[K],
  ) => setConfig((c) => ({ ...c, [key]: value }))

  const cone = useMemo(
    () => (result ? measureLightCone(result) : null),
    [result],
  )

  return (
    <Card>
      <CardHeader>
        <CardTitle>Live Simulator: Time from a Timeless State</CardTitle>
        <CardDescription>
          The Page&ndash;Wootters mechanism. A single timeless global state
          {' '}|Ψ⟩ = Σ&#8342; |k⟩&#8342;&#8342;&#8344;&#8323;&#8342; ⊗ |ψ(t&#8342;)⟩ lives
          on a clock ⊗ system Hilbert space. There is no external time &mdash;
          yet conditioning on a clock reading,
          {' '}|ψ(t&#8342;)⟩ = ⟨k|Ψ⟩, recovers Schrödinger evolution. Stacking
          the emergent spatial slices across clock readings yields an emergent
          spacetime, complete with a light cone.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="space-y-4">
          <div className="grid sm:grid-cols-3 gap-4">
            <label className="text-sm space-y-1 block">
              <span className="text-muted-foreground">
                Chain length: {config.n}
              </span>
              <input
                type="range"
                min={5}
                max={11}
                value={config.n}
                onChange={(e) => update('n', Number(e.target.value))}
                className="w-full accent-primary"
              />
            </label>
            <label className="text-sm space-y-1 block">
              <span className="text-muted-foreground">
                Field h: {config.field.toFixed(1)}
              </span>
              <input
                type="range"
                min={0.4}
                max={2}
                step={0.1}
                value={config.field}
                onChange={(e) => update('field', Number(e.target.value))}
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
                max={28}
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
            {running ? 'Evolving the universe…' : 'Build spacetime'}
          </Button>
        </div>

        {result && cone && (
          <div className="space-y-6 border-t pt-6">
            <div className="flex flex-wrap items-center gap-3">
              <Badge variant="secondary">{result.sites}-site chain</Badge>
              <Badge variant="outline">
                {result.backend === 'wasm' ? 'Rust WASM' : 'TypeScript'}
              </Badge>
              <Badge>
                emergent speed of light v ≈ {cone.velocity.toFixed(2)}{' '}
                sites/time
              </Badge>
              <Badge variant="outline">
                energy drift{' '}
                {Math.max(
                  ...result.slices.map((s) =>
                    Math.abs(s.energy - result.slices[0].energy),
                  ),
                ).toExponential(1)}
              </Badge>
              <span className="text-xs text-muted-foreground">
                unitary · {result.elapsedMs.toFixed(0)} ms
              </span>
            </div>

            <div className="grid lg:grid-cols-2 gap-6">
              <div>
                <h4 className="text-sm font-medium mb-2">
                  Emergent spacetime (light cone)
                </h4>
                <SpacetimeDiagram
                  result={result}
                  cone={cone}
                  selected={selected}
                  onSelect={setSelected}
                />
                <p className="text-xs text-muted-foreground mt-2">
                  Brightness = |⟨Z&#7522;⟩ − reference|, the information spreading
                  from a central defect. The dashed lines are the fitted light
                  cone; its slope is the emergent Lieb&ndash;Robinson velocity.
                </p>
              </div>
              <div className="space-y-3">
                <h4 className="text-sm font-medium">
                  Spatial slice at clock reading k = {selected} (t ={' '}
                  {result.slices[selected].t.toFixed(2)})
                </h4>
                <SpatialSlice result={result} selected={selected} />
                <input
                  type="range"
                  min={0}
                  max={result.slices.length - 1}
                  value={selected}
                  onChange={(e) => setSelected(Number(e.target.value))}
                  className="w-full max-w-[460px] accent-primary"
                />
                <p className="text-xs text-muted-foreground">
                  Each clock reading is a slice ⟨k|Ψ⟩ of the timeless state.
                  Qubits sit at their entanglement-derived positions; brightness
                  marks the propagating disturbance.
                </p>
              </div>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
