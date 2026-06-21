import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { Link } from 'react-router-dom'
import { Play, Pause, Loader2, RotateCcw } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import { UniverseCanvas } from '@/components/universe/UniverseCanvas'
import {
  runUniverse3D,
  stateAtUniverseSlice,
  type Universe3DConfig,
  type Universe3DResult,
} from '@/sim/runner'
import { analyzeEmergentGeometry } from '@/sim/geometry'

type LatticePreset = '2x2x2' | '2x2x3'

const PRESETS: Record<
  LatticePreset,
  { lx: number; ly: number; lz: number; label: string }
> = {
  '2x2x2': { lx: 2, ly: 2, lz: 2, label: '2×2×2 (8 qubits)' },
  '2x2x3': { lx: 2, ly: 2, lz: 3, label: '2×2×3 (12 qubits, slower)' },
}

const DEFAULT: Universe3DConfig & { preset: LatticePreset } = {
  preset: '2x2x2',
  lx: 2,
  ly: 2,
  lz: 2,
  field: 1,
  dt: 0.25,
  steps: 18,
}

function SpacetimeStrip({
  result,
  selected,
  onSelect,
}: {
  result: Universe3DResult
  selected: number
  onSelect: (k: number) => void
}) {
  const rows = result.spacetime.slices.length
  const cols = result.spacetime.sites
  let max = 1e-9
  for (const s of result.spacetime.slices) {
    for (const v of s.signal) max = Math.max(max, v)
  }
  const cell = 10
  const W = cols * cell + 8
  const H = rows * cell + 8

  return (
    <svg
      viewBox={`0 0 ${W} ${H}`}
      className="w-full max-h-40"
      preserveAspectRatio="xMidYMid meet"
      role="img"
      aria-label="Emergent spacetime signal heatmap"
    >
      {result.spacetime.slices.map((slice, k) =>
        slice.signal.map((v, i) => (
          <rect
            key={`${k}-${i}`}
            x={4 + i * cell}
            y={4 + k * cell}
            width={cell - 1}
            height={cell - 1}
            rx={1}
            fill={`oklch(0.72 0.16 290 / ${0.05 + (v / max) * 0.95})`}
            stroke={k === selected ? 'oklch(0.85 0.12 200)' : 'none'}
            strokeWidth={k === selected ? 1.2 : 0}
            onClick={() => onSelect(k)}
            style={{ cursor: 'pointer' }}
          />
        )),
      )}
    </svg>
  )
}

export function UniverseLab() {
  const [config, setConfig] = useState(DEFAULT)
  const [result, setResult] = useState<Universe3DResult | null>(null)
  const [selected, setSelected] = useState(0)
  const [running, setRunning] = useState(false)
  const [playing, setPlaying] = useState(false)
  const timer = useRef<number | null>(null)

  const run = useCallback(() => {
    setRunning(true)
    setPlaying(false)
    setTimeout(() => {
      try {
        const { preset: _, ...cfg } = config
        setResult(runUniverse3D(cfg))
        setSelected(0)
      } finally {
        setRunning(false)
      }
    }, 30)
  }, [config])

  const sliceMi = useMemo(() => {
    if (!result) return null
    const { preset: _, ...cfg } = config
    const { state } = stateAtUniverseSlice(cfg, selected)
    return analyzeEmergentGeometry(state, 1, 3).mi
  }, [result, selected, config])

  const sliceAnalysis = useMemo(() => {
    if (!result) return null
    const { preset: _, ...cfg } = config
    const { state } = stateAtUniverseSlice(cfg, selected)
    return analyzeEmergentGeometry(state, 1, 3)
  }, [result, selected, config])

  useEffect(() => {
    if (!playing || !result) return
    timer.current = window.setInterval(() => {
      setSelected((k) => {
        if (k >= result.spacetime.slices.length - 1) {
          setPlaying(false)
          return k
        }
        return k + 1
      })
    }, 400)
    return () => {
      if (timer.current) clearInterval(timer.current)
    }
  }, [playing, result])

  const setPreset = (preset: LatticePreset) => {
    const p = PRESETS[preset]
    setConfig((c) => ({ ...c, preset, lx: p.lx, ly: p.ly, lz: p.lz }))
  }

  const slice = result?.spacetime.slices[selected]

  return (
    <div className="mx-auto max-w-6xl px-6 py-8 space-y-6">
      <header className="space-y-2">
        <h1 className="text-3xl font-semibold tracking-tight">
          Universe Lab
        </h1>
        <p className="text-muted-foreground max-w-3xl leading-relaxed">
          A minimal <strong className="text-foreground">3+1</strong> toy
          universe: eight qubits on a 3D lattice, emergent spatial positions
          from entanglement (MDS), emergent time from Page&ndash;Wootters clock
          slices. Positions are <em>reconstructed</em>, not built in — emergent
          dim&nbsp;≈&nbsp;3 is a hypothesis we test, not a guarantee at this
          size.{' '}
          <Link to="/" className="underline hover:text-foreground">
            Back to essay
          </Link>
        </p>
      </header>

      <Card>
        <CardHeader>
          <CardTitle>3+1 emergent spacetime</CardTitle>
          <CardDescription>
            Central defect quench on a TFIM cube. Drag to rotate the 3D view;
            use the slider or heatmap to move through emergent time.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="rounded-lg border bg-muted/40 px-4 py-3 text-sm text-muted-foreground leading-relaxed space-y-3">
            <p className="font-medium text-foreground">How time is shown</p>
            <p>
              There is no clock outside this system. Following the
              Page&ndash;Wootters idea, we build a single timeless history
              state and read <em>emergent time</em> off an internal clock:
              slice&nbsp;
              <code className="font-mono text-xs bg-muted px-1 py-0.5 rounded">
                k
              </code>{' '}
              is the quantum state after{' '}
              <code className="font-mono text-xs bg-muted px-1 py-0.5 rounded">
                k
              </code>{' '}
              Schr&ouml;dinger steps of size{' '}
              <code className="font-mono text-xs bg-muted px-1 py-0.5 rounded">
                &Delta;t
              </code>
              , so{' '}
              <code className="font-mono text-xs bg-muted px-1 py-0.5 rounded">
                t = k &middot; &Delta;t
              </code>
              .
            </p>
            <ul className="list-disc list-inside space-y-1.5 pl-1">
              <li>
                <strong className="text-foreground">3D universe view</strong>{' '}
                &mdash; one <em>spatial</em> snapshot at the selected clock
                reading. Node positions come from entanglement (MDS) at that
                instant; glow tracks the defect spreading.
              </li>
              <li>
                <strong className="text-foreground">Slider / Play</strong>{' '}
                &mdash; steps through clock readings{' '}
                <code className="font-mono text-xs">k = 0, 1, 2, …</code>. You
                are not watching motion inside a single frame; you are choosing
                which moment of emergent history to inspect.
              </li>
              <li>
                <strong className="text-foreground">Spacetime heatmap</strong>{' '}
                &mdash; the full{' '}
                <code className="font-mono text-xs">2+1</code> diagram:
                qubits horizontally, emergent time downward, brightness =
                disturbance signal. A bright band spreading down-right is the
                light cone in this discrete universe.
              </li>
            </ul>
            <p>
              So this is honestly <strong className="text-foreground">3+1</strong>
              : three emergent spatial dimensions in the main view, plus one
              emergent time axis on the scrubber and heatmap &mdash; not a
              fourth graphics axis, but a sequence of conditioned states.
            </p>
          </div>
          <div className="flex flex-wrap gap-3 items-center">
            <div className="flex gap-2">
              {(Object.keys(PRESETS) as LatticePreset[]).map((key) => (
                <Button
                  key={key}
                  variant={config.preset === key ? 'default' : 'outline'}
                  size="sm"
                  onClick={() => setPreset(key)}
                >
                  {PRESETS[key].label}
                </Button>
              ))}
            </div>
            <label className="text-sm flex items-center gap-2">
              <span className="text-muted-foreground whitespace-nowrap">
                h = {config.field.toFixed(1)}
              </span>
              <input
                type="range"
                min={0.6}
                max={2}
                step={0.1}
                value={config.field}
                onChange={(e) =>
                  setConfig((c) => ({ ...c, field: Number(e.target.value) }))
                }
                className="w-24 accent-primary"
              />
            </label>
            <Button onClick={run} disabled={running}>
              {running ? (
                <Loader2 className="size-4 animate-spin" />
              ) : (
                <Play className="size-4" />
              )}
              {running ? 'Evolving…' : 'Create universe'}
            </Button>
          </div>

          {result && slice && (
            <div className="grid lg:grid-cols-[1fr_240px] gap-6 items-start">
              <div className="space-y-4 min-w-0">
                <UniverseCanvas
                  result={result}
                  selected={selected}
                  mi={sliceMi}
                />
                <div className="flex items-center gap-3">
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => setPlaying((p) => !p)}
                  >
                    {playing ? (
                      <Pause className="size-4" />
                    ) : (
                      <Play className="size-4" />
                    )}
                    {playing ? 'Pause' : 'Play'}
                  </Button>
                  <Button
                    size="sm"
                    variant="ghost"
                    onClick={() => {
                      setPlaying(false)
                      setSelected(0)
                    }}
                  >
                    <RotateCcw className="size-4" />
                    Reset
                  </Button>
                  <input
                    type="range"
                    min={0}
                    max={result.spacetime.slices.length - 1}
                    value={selected}
                    onChange={(e) => {
                      setPlaying(false)
                      setSelected(Number(e.target.value))
                    }}
                    className="flex-1 accent-primary"
                  />
                  <span className="text-xs text-muted-foreground whitespace-nowrap font-mono">
                    k={selected} t={slice.t.toFixed(2)}
                  </span>
                </div>
              </div>

              <aside className="space-y-4 text-sm">
                <div className="flex flex-wrap gap-2">
                  <Badge variant="secondary">{result.model.label}</Badge>
                  <Badge>
                    dim≈{sliceAnalysis?.mds.emergentDim ?? '?'}
                  </Badge>
                </div>
                <dl className="space-y-2 text-muted-foreground">
                  <div className="flex justify-between gap-2">
                    <dt>Energy</dt>
                    <dd className="font-mono text-foreground">
                      {slice.energy.toFixed(4)}
                    </dd>
                  </div>
                  <div className="flex justify-between gap-2">
                    <dt>Max signal</dt>
                    <dd className="font-mono text-foreground">
                      {Math.max(...slice.signal).toFixed(3)}
                    </dd>
                  </div>
                  <div className="flex justify-between gap-2">
                    <dt>LR velocity</dt>
                    <dd className="font-mono text-foreground">
                      {result.lightCone.velocity.toFixed(2)}
                    </dd>
                  </div>
                  <div className="flex justify-between gap-2">
                    <dt>Energy drift</dt>
                    <dd className="font-mono text-foreground">
                      {result.spacetime.energyDrift.toExponential(1)}
                    </dd>
                  </div>
                  <div className="flex justify-between gap-2">
                    <dt>Defect site</dt>
                    <dd className="font-mono text-foreground">
                      {result.defectSite}
                    </dd>
                  </div>
                </dl>
                <div>
                  <h4 className="text-xs font-medium mb-2 text-foreground">
                    Emergent time (heatmap)
                  </h4>
                  <SpacetimeStrip
                    result={result}
                    selected={selected}
                    onSelect={(k) => {
                      setPlaying(false)
                      setSelected(k)
                    }}
                  />
                  <p className="text-xs text-muted-foreground mt-2">
                    Rows = clock readings (time increases downward). Columns =
                    qubits. Click a row to jump to that slice; the highlighted
                    row matches the 3D view above.
                  </p>
                </div>
                <p className="text-xs text-muted-foreground leading-relaxed">
                  Edge opacity = mutual information at this slice. Node glow =
                  light-cone signal from the central defect.
                </p>
              </aside>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
