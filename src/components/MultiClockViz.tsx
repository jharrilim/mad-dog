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
  runMultiClockAsync,
  type MultiClockResultWithBackend,
} from '@/sim/runner-async'
import type { MultiClockConfig } from '@/sim/types'

const LATTICE_PRESETS = {
  chain: {
    label: 'Chain n=9',
    config: {
      kind: 'chain' as const,
      n: 9,
      field: 1,
      dt: 0.2,
      steps: 40,
      physicalSlices: 15,
    },
    preset: 3 as PresetKey,
  },
  grid: {
    label: 'Grid 3×3',
    config: {
      kind: 'grid' as const,
      rows: 3,
      cols: 3,
      field: 1,
      dt: 0.2,
      steps: 40,
      physicalSlices: 15,
    },
    preset: 3 as PresetKey,
  },
  cube: {
    label: 'Cube 2×2×3',
    config: {
      kind: 'cube' as const,
      rows: 2,
      cols: 2,
      lz: 3,
      field: 1,
      dt: 0.2,
      steps: 40,
      physicalSlices: 15,
    },
    preset: 3 as PresetKey,
  },
} as const

type LatticeKey = keyof typeof LATTICE_PRESETS

const PRESETS = {
  3: { label: '3 clocks (corners + defect)', sites: (n: number) => [0, Math.floor(n / 2), n - 1] },
  5: {
    label: '5 clocks (spread)',
    sites: (n: number) => [0, Math.floor(n / 4), Math.floor(n / 2), Math.floor((3 * n) / 4), n - 1],
  },
} as const

type PresetKey = keyof typeof PRESETS

function r2Color(r2: number): string {
  if (r2 >= 0.95) return 'oklch(0.72 0.14 155 / 0.85)'
  if (r2 >= 0.85) return 'oklch(0.78 0.13 60 / 0.85)'
  return 'oklch(0.72 0.16 290 / 0.85)'
}

function PairwiseMatrix({ result }: { result: MultiClockResultWithBackend }) {
  const n = result.labels.length
  const cell = Math.min(52, Math.max(36, 280 / n))
  const pad = 88
  const W = pad + n * cell + 8
  const H = pad + n * cell + 8

  return (
    <svg viewBox={`0 0 ${W} ${H}`} className="w-full h-auto max-w-lg" role="img">
      {result.labels.map((label, j) => (
        <text
          key={`col-${j}`}
          x={pad + j * cell + cell / 2}
          y={pad - 8}
          textAnchor="middle"
          className="fill-muted-foreground text-[8px]"
        >
          {label.length > 14 ? `…${label.slice(-12)}` : label}
        </text>
      ))}
      {result.labels.map((label, i) => (
        <text
          key={`row-${i}`}
          x={pad - 6}
          y={pad + i * cell + cell / 2 + 3}
          textAnchor="end"
          className="fill-muted-foreground text-[8px]"
        >
          {label.length > 12 ? label.replace('Site ', 's') : label}
        </text>
      ))}
      {result.pairwiseR2.map((row, i) =>
        row.map((r2, j) => (
          <g key={`${i}-${j}`}>
            <rect
              x={pad + j * cell + 1}
              y={pad + i * cell + 1}
              width={cell - 2}
              height={cell - 2}
              rx={3}
              fill={i === j ? 'oklch(0.55 0.02 280 / 0.35)' : r2Color(r2)}
            />
            <text
              x={pad + j * cell + cell / 2}
              y={pad + i * cell + cell / 2 + 3}
              textAnchor="middle"
              className="fill-foreground text-[9px] font-mono"
            >
              {i === j ? '—' : r2.toFixed(2)}
            </text>
          </g>
        )),
      )}
    </svg>
  )
}

type RunConfig = MultiClockConfig & { preset: PresetKey; lattice: LatticeKey }

const DEFAULT: RunConfig = {
  ...LATTICE_PRESETS.chain.config,
  preset: 3,
  lattice: 'chain',
}

export function MultiClockViz() {
  const [config, setConfig] = useState<RunConfig>(DEFAULT)
  const [result, setResult] = useState<MultiClockResultWithBackend | null>(null)
  const [running, setRunning] = useState(false)

  const sites = result?.sites ?? config.n ?? (config.rows ?? 3) * (config.cols ?? 3)

  const clockSites = useMemo(
    () => PRESETS[config.preset].sites(sites),
    [config.preset, sites],
  )

  const run = useCallback(() => {
    setRunning(true)
    const { preset: _p, lattice: _l, ...runConfig } = config
    void runMultiClockAsync({
      ...runConfig,
      clockSites: config.kind === 'chain' ? clockSites : undefined,
    })
      .then(setResult)
      .finally(() => setRunning(false))
  }, [config, clockSites])

  const applyLattice = (key: LatticeKey) => {
    const preset = LATTICE_PRESETS[key]
    setConfig((c) => ({
      ...c,
      ...preset.config,
      lattice: key,
      preset: preset.preset,
    }))
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Multi-clock consistency network</CardTitle>
        <CardDescription>
          One quenched trajectory, many physical clocks — each site ticks when
          local disturbance signal crosses thresholds. The matrix shows pairwise
          sync R²: do all clocks agree on &ldquo;same state, same time
          reading&rdquo;? A globally consistent time requires high R² everywhere;
          relational time predicts they will not all match.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="flex flex-wrap items-center gap-4">
          <div className="flex gap-2">
            {(Object.keys(LATTICE_PRESETS) as LatticeKey[]).map((key) => (
              <Button
                key={key}
                size="sm"
                variant={config.lattice === key ? 'default' : 'outline'}
                onClick={() => applyLattice(key)}
              >
                {LATTICE_PRESETS[key].label}
              </Button>
            ))}
          </div>
          {config.kind === 'chain' && (
            <>
              <div className="flex gap-2">
                {(Object.keys(PRESETS) as unknown as PresetKey[]).map((key) => (
                  <Button
                    key={key}
                    size="sm"
                    variant={config.preset === key ? 'default' : 'outline'}
                    onClick={() => setConfig((c) => ({ ...c, preset: key }))}
                  >
                    {PRESETS[key].label}
                  </Button>
                ))}
              </div>
              <label className="text-sm flex items-center gap-2">
                <span className="text-muted-foreground">n = {config.n ?? 9}</span>
                <input
                  type="range"
                  min={7}
                  max={13}
                  value={config.n ?? 9}
                  onChange={(e) =>
                    setConfig((c) => ({ ...c, n: Number(e.target.value) }))
                  }
                  className="w-24 accent-primary"
                />
              </label>
            </>
          )}
          <Button onClick={run} disabled={running} size="sm">
            {running ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : (
              <Play className="mr-2 h-4 w-4" />
            )}
            Build clock network
          </Button>
        </div>

        {result && (
          <>
            <div className="flex flex-wrap gap-2 text-xs">
              <Badge variant="outline">{result.label}</Badge>
              <Badge variant={result.minPairwiseR2 >= 0.95 ? 'outline' : 'default'}>
                min pairwise R² = {result.minPairwiseR2.toFixed(3)}
              </Badge>
              <Badge variant="secondary">
                defect ↔ uniform R² = {result.defectUniformR2.toFixed(3)}
              </Badge>
              <Badge variant="outline">
                edge ↔ edge R² = {result.edgeEdgeR2.toFixed(3)}
              </Badge>
              <Badge variant="outline">
                {result.inconsistentPairs} pairs below 0.95
              </Badge>
              <Badge variant="outline">
                {result.elapsedMs.toFixed(0)} ms · {result.backend}
              </Badge>
            </div>
            <div className="grid lg:grid-cols-2 gap-6 items-start">
              <div>
                <h4 className="text-sm font-medium mb-2">Pairwise sync R²</h4>
                <PairwiseMatrix result={result} />
                <p className="text-xs text-muted-foreground mt-2">
                  Row/column 0 is the uniform Δt clock. Greenish cells (R² ≥
                  0.95) are nearly synchronized; purple cells mark clocks that
                  disagree on the time coordinate for the same underlying state.
                </p>
              </div>
              <div className="space-y-3 text-sm">
                <h4 className="font-medium">Per-clock vs uniform</h4>
                <ul className="space-y-2 text-muted-foreground">
                  {result.clocks.map((c) => (
                    <li
                      key={c.site}
                      className="flex justify-between gap-2 border-b border-border/50 pb-2"
                    >
                      <span>{c.label}</span>
                      <span className="font-mono text-foreground">
                        R² = {c.syncR2VsUniform.toFixed(3)}
                      </span>
                    </li>
                  ))}
                </ul>
                <p className="text-xs text-muted-foreground leading-relaxed">
                  The defect-site clock stays synced with the uniform clock
                  (local physics matches the global tick). Edge clocks stall
                  then catch up as the light cone arrives — so the network is
                  not globally transitive: A agrees with uniform, B agrees with
                  uniform, but A and B can still disagree with each other mid-quench.
                </p>
              </div>
            </div>
          </>
        )}
      </CardContent>
    </Card>
  )
}
