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
import { runQeccProbeAsync } from '@/sim/runner-async'
import type { ExcitationSubspaceConfig, QeccProbeResult } from '@/sim/types'

type Result = QeccProbeResult & { backend: string }
type ModelPreset = 'tfim' | 'xx' | 'heisenberg'

const PRESETS: Record<ModelPreset, ExcitationSubspaceConfig> = {
  tfim: {
    n: 8,
    field: 1.2,
    dt: 0.2,
    steps: 22,
    coupleStep: 7,
    coupling: 0.9,
    seed: 4242,
    windowRadius: 2,
    model: 'tfim',
  },
  xx: {
    n: 8,
    field: 1.2,
    dt: 0.2,
    steps: 22,
    coupleStep: 7,
    coupling: 0.9,
    seed: 4242,
    windowRadius: 2,
    model: 'xx',
  },
  heisenberg: {
    n: 6,
    field: 2.0,
    dt: 0.2,
    steps: 22,
    coupleStep: 5,
    coupling: 0.85,
    seed: 4242,
    windowRadius: 3,
    model: 'heisenberg',
  },
}

function FidelityChart({ result }: { result: Result }) {
  const W = 320
  const H = 120
  const pad = { l: 48, r: 12, t: 12, b: 28 }
  const labels = ['Branch 0', 'Branch 1', 'Mixed']
  const values = [
    result.qecc.branch0CodeFidelity,
    result.qecc.branch1CodeFidelity,
    result.qecc.mixedCodeFidelity,
  ]
  const max = Math.max(...values, 0.05)
  const barW = (W - pad.l - pad.r) / labels.length - 8

  return (
    <svg viewBox={`0 0 ${W} ${H}`} className="w-full max-w-md h-auto" role="img">
      <line
        x1={pad.l}
        y1={H - pad.b - (0.5 / max) * (H - pad.t - pad.b)}
        x2={W - pad.r}
        y2={H - pad.b - (0.5 / max) * (H - pad.t - pad.b)}
        stroke="oklch(0.55 0.08 150)"
        strokeDasharray="4 3"
        strokeWidth={1}
      />
      {labels.map((label, i) => {
        const h = ((values[i] ?? 0) / max) * (H - pad.t - pad.b)
        const x = pad.l + i * (barW + 8)
        const y = H - pad.b - h
        return (
          <g key={label}>
            <rect
              x={x}
              y={y}
              width={barW}
              height={h}
              rx={3}
              fill={i < 2 ? 'oklch(0.62 0.14 290)' : 'oklch(0.55 0.08 250)'}
            />
            <text
              x={x + barW / 2}
              y={H - 8}
              textAnchor="middle"
              className="fill-muted-foreground text-[9px]"
            >
              {label}
            </text>
            <text
              x={x + barW / 2}
              y={y - 3}
              textAnchor="middle"
              className="fill-foreground text-[8px]"
            >
              {(values[i] ?? 0).toFixed(2)}
            </text>
          </g>
        )
      })}
      <text
        x={pad.l - 4}
        y={H - pad.b - (0.5 / max) * (H - pad.t - pad.b) + 4}
        textAnchor="end"
        className="fill-muted-foreground text-[8px]"
      >
        0.5
      </text>
    </svg>
  )
}

export function QeccViz() {
  const [preset, setPreset] = useState<ModelPreset>('tfim')
  const [result, setResult] = useState<Result | null>(null)
  const [loading, setLoading] = useState(false)

  const run = useCallback(() => {
    setLoading(true)
    void runQeccProbeAsync(PRESETS[preset])
      .then(setResult)
      .finally(() => setLoading(false))
  }, [preset])

  return (
    <Card>
      <CardHeader>
        <CardTitle>QECC code subspace probe</CardTitle>
        <CardDescription>
          After decoherence, do Everett branches reside in a distinguishable code
          subspace? Works on TFIM, XX, and Heisenberg chains — same env coupling,
          different bulk Hamiltonians.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex flex-wrap gap-2">
          {(['tfim', 'xx', 'heisenberg'] as const).map((m) => (
            <Button
              key={m}
              size="sm"
              variant={preset === m ? 'default' : 'outline'}
              onClick={() => setPreset(m)}
            >
              {m === 'tfim' ? 'TFIM' : m === 'xx' ? 'XX' : 'Heisenberg'}
            </Button>
          ))}
        </div>
        <Button onClick={run} disabled={loading} size="sm">
          {loading ? (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          ) : (
            <Play className="mr-2 h-4 w-4" />
          )}
          Run probe
        </Button>
        {result && (
          <>
            <div className="flex flex-wrap gap-2">
              <Badge variant="outline">{result.excitation.model}</Badge>
              <Badge variant={result.qecc.codeSubspaceFound ? 'default' : 'secondary'}>
                {result.qecc.codeSubspaceFound ? 'Code subspace found' : 'Not found'}
              </Badge>
              <Badge variant="outline">{result.qecc.codeLabel}</Badge>
              <Badge variant="outline">
                sel {result.qecc.fidelitySelectivity.toFixed(2)}×
              </Badge>
              <Badge variant="outline">
                avg fidelity {result.qecc.branchAvgFidelity.toFixed(3)}
              </Badge>
            </div>
            <p className="text-xs text-muted-foreground">
              Window {result.excitation.windowSites.join(', ')} —{' '}
              {result.stabilizer.generatorCount} generators found (d={result.qecc.dDistance}).
              Selectivity = branch avg / mixed fidelity; dashed line marks 50%.
            </p>
            <FidelityChart result={result} />
          </>
        )}
      </CardContent>
    </Card>
  )
}
