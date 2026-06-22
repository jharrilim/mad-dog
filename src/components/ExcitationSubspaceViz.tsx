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
  runExcitationSubspaceAsync,
  type ExcitationSubspaceResultWithBackend,
} from '@/sim/runner-async'

function SharpnessChart({ result }: { result: ExcitationSubspaceResultWithBackend }) {
  const W = 320
  const H = 120
  const pad = { l: 48, r: 12, t: 12, b: 28 }
  const labels = ['Mixed', 'Branch 0', 'Branch 1']
  const values = [
    result.mixedSharpness,
    result.branch0Sharpness,
    result.branch1Sharpness,
  ]
  const max = Math.max(...values, 0.05)
  const barW = (W - pad.l - pad.r) / labels.length - 8

  return (
    <svg viewBox={`0 0 ${W} ${H}`} className="w-full max-w-md h-auto" role="img">
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
              fill={i === 0 ? 'oklch(0.55 0.08 250)' : 'oklch(0.62 0.14 290)'}
            />
            <text
              x={x + barW / 2}
              y={H - 8}
              textAnchor="middle"
              className="fill-muted-foreground text-[9px]"
            >
              {label}
            </text>
          </g>
        )
      })}
    </svg>
  )
}

export function ExcitationSubspaceViz() {
  const [result, setResult] = useState<ExcitationSubspaceResultWithBackend | null>(null)
  const [loading, setLoading] = useState(false)

  const run = useCallback(() => {
    setLoading(true)
    void runExcitationSubspaceAsync({
      n: 8,
      field: 1.2,
      dt: 0.2,
      steps: 22,
      coupleStep: 7,
      coupling: 0.9,
      seed: 4242,
      windowRadius: 2,
    })
      .then(setResult)
      .finally(() => setLoading(false))
  }, [])

  return (
    <Card>
      <CardHeader>
        <CardTitle>Excitation subspace probe (QECC / EFT)</CardTitle>
        <CardDescription>
          After environment decoherence, do branch-resolved excitations look like a
          low-rank, Pauli-sharp subspace around the tracked defect? Compares mixed
          chain marginal vs conditional branches on the excitation window.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
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
              <Badge variant={result.codeLike ? 'default' : 'secondary'}>
                {result.codeLike ? 'Code-like' : 'Not code-like'}
              </Badge>
              <Badge variant="outline">
                gain {result.sharpnessGain.toFixed(2)}×
              </Badge>
              <Badge variant="outline">
                rank ratio {result.rankReduction.toFixed(2)}
              </Badge>
              <Badge variant="outline">
                branch overlap {result.branchOverlap.toFixed(3)}
              </Badge>
            </div>
            <p className="text-xs text-muted-foreground">
              Window sites {result.windowSites.join(', ')} (peak {result.excitationPeak});
              env weights p₀={result.envP0.toFixed(2)} p₁={result.envP1.toFixed(2)}.
            </p>
            <SharpnessChart result={result} />
          </>
        )}
      </CardContent>
    </Card>
  )
}
