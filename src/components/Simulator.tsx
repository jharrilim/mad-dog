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
  type ModelKind,
  type RunConfig,
} from '@/sim/runner'
import { runEmergenceAsync, type RunResultWithBackend } from '@/sim/runner-async'

const MODELS: { kind: ModelKind; label: string; blurb: string }[] = [
  {
    kind: 'chain',
    label: '1D chain',
    blurb: 'Local Ising chain — expect a 1D line to emerge',
  },
  {
    kind: 'grid',
    label: '2D grid',
    blurb: 'Local Ising grid — expect a 2D sheet to emerge',
  },
  {
    kind: 'random',
    label: 'Random non-local',
    blurb: 'All-to-all couplings — no clean low-D space should emerge',
  },
]

function MiHeatmap({ mi }: { mi: number[][] }) {
  const n = mi.length
  let max = 0
  for (let i = 0; i < n; i++)
    for (let j = 0; j < n; j++) if (i !== j) max = Math.max(max, mi[i][j])
  const cell = 22
  const pad = 4
  const size = n * cell + pad * 2
  return (
    <svg
      viewBox={`0 0 ${size} ${size}`}
      className="w-full h-auto max-w-[260px]"
      role="img"
      aria-label="Mutual information heatmap"
    >
      {mi.map((row, i) =>
        row.map((value, j) => {
          const intensity = i === j ? 0 : max > 0 ? value / max : 0
          return (
            <rect
              key={`${i}-${j}`}
              x={pad + j * cell}
              y={pad + i * cell}
              width={cell - 1}
              height={cell - 1}
              rx={2}
              fill="oklch(0.72 0.16 290)"
              fillOpacity={i === j ? 0.08 : 0.12 + intensity * 0.88}
            />
          )
        }),
      )}
    </svg>
  )
}

function MdsEmbedding({
  coords,
  mi,
}: {
  coords: number[][]
  mi: number[][]
}) {
  const n = coords.length
  const pts = coords.map((c) => ({ x: c[0] ?? 0, y: c[1] ?? 0 }))
  const xs = pts.map((p) => p.x)
  const ys = pts.map((p) => p.y)
  const minX = Math.min(...xs)
  const maxX = Math.max(...xs)
  const minY = Math.min(...ys)
  const maxY = Math.max(...ys)
  const cx = (minX + maxX) / 2
  const cy = (minY + maxY) / 2
  const W = 300
  const H = 240
  const m = 28
  // Uniform scale on both axes: MDS distances are isotropic, so we must not
  // stretch dimensions independently. Center the cloud in the viewport.
  const span = Math.max(maxX - minX, maxY - minY) || 1
  const scale = Math.min(W - 2 * m, H - 2 * m) / span
  const sx = (x: number) => W / 2 + (x - cx) * scale
  const sy = (y: number) => H / 2 - (y - cy) * scale

  let miMax = 0
  for (let i = 0; i < n; i++)
    for (let j = i + 1; j < n; j++) miMax = Math.max(miMax, mi[i][j])

  return (
    <svg
      viewBox={`0 0 ${W} ${H}`}
      className="w-full h-auto"
      role="img"
      aria-label="Emergent geometry (MDS embedding)"
    >
      {/* Edges weighted by mutual information */}
      {Array.from({ length: n }).map((_, i) =>
        Array.from({ length: n }).map((__, j) => {
          if (j <= i) return null
          const strength = miMax > 0 ? mi[i][j] / miMax : 0
          if (strength < 0.08) return null
          return (
            <line
              key={`e-${i}-${j}`}
              x1={sx(pts[i].x)}
              y1={sy(pts[i].y)}
              x2={sx(pts[j].x)}
              y2={sy(pts[j].y)}
              stroke="oklch(0.65 0.14 200)"
              strokeWidth={0.5 + strength * 3}
              strokeOpacity={0.15 + strength * 0.5}
            />
          )
        }),
      )}
      {pts.map((p, i) => (
        <g key={`n-${i}`}>
          <circle
            cx={sx(p.x)}
            cy={sy(p.y)}
            r={9}
            className="fill-card stroke-primary"
            strokeWidth={1.5}
          />
          <text
            x={sx(p.x)}
            y={sy(p.y) + 3.5}
            textAnchor="middle"
            className="fill-foreground text-[9px] font-mono"
          >
            {i}
          </text>
        </g>
      ))}
    </svg>
  )
}

function ScreePlot({
  eigenvalues,
  emergentDim,
}: {
  eigenvalues: number[]
  emergentDim: number
}) {
  const top = eigenvalues.slice(0, 8).map((v) => Math.max(0, v))
  const max = Math.max(...top, 1e-9)
  const W = 300
  const H = 200
  const m = 28
  const barW = (W - 2 * m) / top.length
  return (
    <svg
      viewBox={`0 0 ${W} ${H}`}
      className="w-full h-auto"
      role="img"
      aria-label="MDS eigenvalue spectrum"
    >
      <line x1={m} y1={H - m} x2={W - m} y2={H - m} className="stroke-border" />
      {top.map((v, i) => {
        const h = (v / max) * (H - 2 * m)
        const within = i < emergentDim
        return (
          <g key={i}>
            <rect
              x={m + i * barW + barW * 0.15}
              y={H - m - h}
              width={barW * 0.7}
              height={h}
              rx={2}
              fill={
                within ? 'oklch(0.72 0.16 290)' : 'oklch(0.4 0.03 280)'
              }
            />
            <text
              x={m + i * barW + barW * 0.5}
              y={H - m + 12}
              textAnchor="middle"
              className="fill-muted-foreground text-[9px]"
            >
              {i + 1}
            </text>
          </g>
        )
      })}
      <text x={m} y={16} className="fill-muted-foreground text-[10px]">
        eigenvalue
      </text>
      <text
        x={W - m}
        y={H - 6}
        textAnchor="end"
        className="fill-muted-foreground text-[10px]"
      >
        dimension index
      </text>
    </svg>
  )
}

const DEFAULT_CONFIG: RunConfig = {
  kind: 'chain',
  n: 8,
  rows: 3,
  cols: 3,
  field: 1.5,
  seed: 12345,
}

export function Simulator() {
  const [config, setConfig] = useState<RunConfig>(DEFAULT_CONFIG)
  const [result, setResult] = useState<RunResultWithBackend | null>(null)
  const [running, setRunning] = useState(false)

  const qubitCount = useMemo(() => {
    if (config.kind === 'grid') return (config.rows ?? 3) * (config.cols ?? 3)
    if (config.kind === 'cube') {
      return (config.lx ?? 2) * (config.ly ?? 2) * (config.lz ?? 2)
    }
    return config.n ?? 8
  }, [config])

  const run = useCallback(() => {
    setRunning(true)
    void runEmergenceAsync(config)
      .then(setResult)
      .finally(() => setRunning(false))
  }, [config])

  const update = <K extends keyof RunConfig>(key: K, value: RunConfig[K]) =>
    setConfig((c) => ({ ...c, [key]: value }))

  return (
    <Card>
      <CardHeader>
        <CardTitle>Live Simulator: Space from Hilbert Space</CardTitle>
        <CardDescription>
          Find a low-energy state of a chosen Hamiltonian, compute the
          mutual-information geometry between qubits, and detect the emergent
          spatial dimension via classical MDS. Local Hamiltonians yield clean
          low-dimensional space; non-local ones do not.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Controls */}
        <div className="space-y-4">
          <div className="flex flex-wrap gap-2">
            {MODELS.map((m) => (
              <Button
                key={m.kind}
                variant={config.kind === m.kind ? 'default' : 'outline'}
                size="sm"
                onClick={() => update('kind', m.kind)}
              >
                {m.label}
              </Button>
            ))}
          </div>
          <p className="text-xs text-muted-foreground">
            {MODELS.find((m) => m.kind === config.kind)?.blurb}
          </p>

          <div className="grid sm:grid-cols-2 gap-4">
            {config.kind === 'grid' ? (
              <>
                <label className="text-sm space-y-1 block">
                  <span className="text-muted-foreground">
                    Rows: {config.rows}
                  </span>
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
                  <span className="text-muted-foreground">
                    Cols: {config.cols}
                  </span>
                  <input
                    type="range"
                    min={2}
                    max={4}
                    value={config.cols}
                    onChange={(e) => update('cols', Number(e.target.value))}
                    className="w-full accent-primary"
                  />
                </label>
              </>
            ) : (
              <label className="text-sm space-y-1 block">
                <span className="text-muted-foreground">
                  Qubits: {config.n}
                </span>
                <input
                  type="range"
                  min={4}
                  max={11}
                  value={config.n}
                  onChange={(e) => update('n', Number(e.target.value))}
                  className="w-full accent-primary"
                />
              </label>
            )}

            {config.kind !== 'random' && (
              <label className="text-sm space-y-1 block">
                <span className="text-muted-foreground">
                  Transverse field h: {config.field.toFixed(2)}{' '}
                  {config.field > 1 ? '(paramagnetic)' : '(ordered)'}
                </span>
                <input
                  type="range"
                  min={0.2}
                  max={2.5}
                  step={0.1}
                  value={config.field}
                  onChange={(e) => update('field', Number(e.target.value))}
                  className="w-full accent-primary"
                />
              </label>
            )}
          </div>

          <div className="flex items-center gap-3">
            <Button onClick={run} disabled={running}>
              {running ? (
                <Loader2 className="size-4 animate-spin" />
              ) : (
                <Play className="size-4" />
              )}
              {running ? 'Solving…' : 'Run simulation'}
            </Button>
            <span className="text-xs text-muted-foreground">
              Hilbert space dimension: 2^{qubitCount} = {2 ** qubitCount}
            </span>
          </div>
          {qubitCount >= 11 && (
            <p className="text-xs text-amber-500/80">
              Large systems may take a few seconds in the WASM worker; the page
              stays responsive.
            </p>
          )}
        </div>

        {/* Results */}
        {result && (
          <div className="space-y-6 border-t pt-6">
            <div className="flex flex-wrap items-center gap-3">
              <Badge variant="secondary">{result.label}</Badge>
              <Badge variant="outline">
                {result.backend === 'wasm' ? 'Rust WASM' : 'TypeScript'}
              </Badge>
              <Badge>
                Emergent dimension: {result.report.mds.emergentDim}
              </Badge>
              <span className="text-xs text-muted-foreground">
                ground energy ≈ {result.energy.toFixed(3)} · {result.iters}{' '}
                iters · {result.elapsedMs.toFixed(0)} ms
              </span>
            </div>

            <div className="grid md:grid-cols-2 gap-6">
              <div>
                <h4 className="text-sm font-medium mb-2">
                  Mutual information I(i:j)
                </h4>
                <MiHeatmap mi={result.report.mi} />
                <p className="text-xs text-muted-foreground mt-2">
                  Brighter = more entangled. This is the only input to the
                  geometry.
                </p>
              </div>
              <div>
                <h4 className="text-sm font-medium mb-2">
                  Emergent geometry (MDS)
                </h4>
                <MdsEmbedding
                  coords={result.report.mds.coords}
                  mi={result.report.mi}
                />
                <p className="text-xs text-muted-foreground mt-2">
                  Qubits placed by entanglement distance. Edges = strong mutual
                  information.
                </p>
              </div>
            </div>

            <div>
              <h4 className="text-sm font-medium mb-2">
                Dimension detector (MDS eigenvalue spectrum)
              </h4>
              <div className="max-w-md">
                <ScreePlot
                  eigenvalues={result.report.mds.eigenvalues}
                  emergentDim={result.report.mds.emergentDim}
                />
              </div>
              <p className="text-xs text-muted-foreground mt-2">
                Highlighted bars span the detected emergent dimension. A sharp
                drop-off means space emerged cleanly; a slow decay means it did
                not.
              </p>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
