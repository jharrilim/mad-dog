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
  runDecoherenceQuenchAsync,
  type DecoherenceQuenchResultWithBackend,
} from '@/sim/runner-async'
import {
  separationChartPath,
  worldlinePolylinePoints,
  WORLDLINE_COLORS,
} from '@/components/worldline-viz'

const MIXED_WORLDLINE_STROKE = 'oklch(0.72 0.04 250 / 0.75)'

function signalColor(v: number, max: number) {
  return `oklch(0.72 0.16 290 / ${0.06 + (v / max) * 0.94})`
}

function DecoherenceDiagram({ result }: { result: DecoherenceQuenchResultWithBackend }) {
  const rows = result.slices.length
  const cols = result.chainSites
  let max = 1e-9
  for (const s of result.slices) {
    for (const v of s.signal) max = Math.max(max, v)
  }

  const cell = 24
  const padL = 40
  const padT = 16
  const W = padL + cols * cell + 8
  const H = padT + rows * cell + 8
  const layout = { cell, padL, padT }
  const coupleY = padT + result.coupleStep * cell

  return (
    <svg
      viewBox={`0 0 ${W} ${H}`}
      className="w-full h-auto max-w-[460px]"
      role="img"
      aria-label="Decoherence quench with branch-resolved worldlines"
    >
      <text x={padL} y={12} className="fill-muted-foreground text-[10px]">
        chain site →
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
            fill={signalColor(v, max)}
          />
        )),
      )}
      <line
        x1={padL}
        y1={coupleY}
        x2={W - 4}
        y2={coupleY}
        stroke="oklch(0.85 0.14 90)"
        strokeWidth={1.5}
        strokeDasharray="5 4"
      />
      <text x={W - 6} y={coupleY - 4} textAnchor="end" className="fill-muted-foreground text-[9px]">
        env couples
      </text>
      {result.mixedWorldline.length > 1 && (
        <polyline
          fill="none"
          stroke={MIXED_WORLDLINE_STROKE}
          strokeWidth={2}
          strokeDasharray="5 4"
          points={worldlinePolylinePoints(result.mixedWorldline, layout)}
        />
      )}
      {result.branches.map((branch, i) =>
        branch.worldline.length > 1 ? (
          <polyline
            key={branch.envBit}
            fill="none"
            stroke={WORLDLINE_COLORS[i] ?? WORLDLINE_COLORS[0]}
            strokeWidth={2.5}
            points={worldlinePolylinePoints(branch.worldline, layout)}
          />
        ) : null,
      )}
    </svg>
  )
}

function EnvEntropyChart({ result }: { result: DecoherenceQuenchResultWithBackend }) {
  const series = result.slices.map((s) => s.envEntropy)
  const W = 300
  const H = 80
  const { line } = separationChartPath(series, W, H)
  const coupleX =
    8 + (result.coupleStep / Math.max(result.slices.length - 1, 1)) * (W - 16)

  return (
    <div className="space-y-1">
      <h4 className="text-sm font-medium">Environment branch entropy</h4>
      <svg viewBox={`0 0 ${W} ${H}`} className="w-full max-w-sm h-auto" role="img">
        <line x1={8} y1={H - 8} x2={W - 8} y2={H - 8} className="stroke-border" />
        <line x1={8} y1={8} x2={8} y2={H - 8} className="stroke-border" />
        <polyline fill="none" stroke={WORLDLINE_COLORS[1]} strokeWidth={2} points={line} />
        <line
          x1={coupleX}
          y1={8}
          x2={coupleX}
          y2={H - 8}
          stroke="oklch(0.85 0.14 90)"
          strokeWidth={1}
          strokeDasharray="4 3"
        />
        <text x={W - 8} y={H - 2} textAnchor="end" className="fill-muted-foreground text-[9px]">
          time →
        </text>
      </svg>
      <p className="text-xs text-muted-foreground">
        S_env rises after coupling — the pointer qubit records which-path information and
        splits the wave function into distinguishable Everett branches.
      </p>
    </div>
  )
}

export function DecoherenceQuenchViz() {
  const [result, setResult] = useState<DecoherenceQuenchResultWithBackend | null>(null)
  const [loading, setLoading] = useState(false)

  const run = useCallback(async () => {
    setLoading(true)
    try {
      const r = await runDecoherenceQuenchAsync({
        n: 8,
        field: 1.2,
        dt: 0.2,
        steps: 22,
        coupleStep: 7,
        coupling: 0.9,
        seed: 4242,
      })
      setResult(r)
    } finally {
      setLoading(false)
    }
  }, [])

  return (
    <Card>
      <CardHeader>
        <CardTitle>Decoherence quench — branch-resolved worldlines</CardTitle>
        <CardDescription>
          A TFIM chain defect quench plus one environment qubit. After a coupling step
          (ZX interaction + CNOT), conditional tracks on each env branch sharpen relative
          to the mixed (pre-branch) centroid — a minimal Everett branching demo.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <Button onClick={run} disabled={loading}>
          {loading ? (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          ) : (
            <Play className="mr-2 h-4 w-4" />
          )}
          Run decoherence quench
        </Button>

        {result && (
          <>
            <div className="flex flex-wrap gap-2 text-xs">
              <Badge variant="outline">{result.elapsedMs.toFixed(0)} ms · {result.backend}</Badge>
              <Badge variant={result.branchesDistinguishable ? 'secondary' : 'destructive'}>
                {result.branchesDistinguishable ? 'branches resolved' : 'weak branching'}
              </Badge>
              <Badge variant="outline">sharpen ×{result.sharpenRatio.toFixed(2)}</Badge>
              <Badge variant="outline">
                p₀={(result.branches[0]?.weight ?? 0).toFixed(2)} p₁=
                {(result.branches[1]?.weight ?? 0).toFixed(2)}
              </Badge>
            </div>

            <div className="grid sm:grid-cols-2 gap-4 items-start">
              <DecoherenceDiagram result={result} />
              <EnvEntropyChart result={result} />
            </div>

            <p className="text-xs text-muted-foreground leading-relaxed">
              Dashed curve: mixed-state peak track (interference smear). Solid curves:
              branch-conditional tracks after environment entanglement. This is not a full
              QECC / infrared subspace identification — see open questions in the essay.
            </p>
          </>
        )}
      </CardContent>
    </Card>
  )
}
