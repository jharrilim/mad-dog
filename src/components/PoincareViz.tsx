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
import { runPoincareCompositeAsync } from '@/sim/runner-async'
import type { PoincareCompositeResult } from '@/sim/types'

type Result = PoincareCompositeResult & { backend: string }

function PillarRow({
  label,
  sublabel,
  ok,
  value,
}: {
  label: string
  sublabel: string
  ok: boolean
  value: string
}) {
  return (
    <div className="flex items-center justify-between py-1.5 border-b last:border-0">
      <div>
        <span className="text-sm font-medium">{label}</span>
        <span className="ml-2 text-xs text-muted-foreground">{sublabel}</span>
      </div>
      <div className="flex items-center gap-2">
        <span className="text-xs text-muted-foreground font-mono">{value}</span>
        <Badge variant={ok ? 'default' : 'secondary'} className="text-[10px] px-1.5">
          {ok ? 'pass' : 'fail'}
        </Badge>
      </div>
    </div>
  )
}

function BoostFrameChart({ result }: { result: Result }) {
  const frames = result.boostFrames
  if (!frames.length) return null
  const W = 320
  const H = 100
  const pad = { l: 52, r: 12, t: 12, b: 28 }
  const max = Math.max(...frames.map(f => Math.abs(f.velocity)), 0.05) * 1.15
  const barW = (W - pad.l - pad.r) / frames.length - 6

  return (
    <svg viewBox={`0 0 ${W} ${H}`} className="w-full max-w-md h-auto" role="img">
      {frames.map((frame, i) => {
        const v = Math.abs(frame.velocity)
        const h = (v / max) * (H - pad.t - pad.b)
        const x = pad.l + i * (barW + 6)
        const y = H - pad.b - h
        return (
          <g key={frame.label}>
            <rect x={x} y={y} width={barW} height={h} rx={3}
              fill="oklch(0.62 0.14 290)" />
            <text x={x + barW / 2} y={H - 8} textAnchor="middle"
              className="fill-muted-foreground text-[8px]">
              {frame.label.replace('corner-', '').replace('edge-', '')}
            </text>
            <text x={x + barW / 2} y={y - 3} textAnchor="middle"
              className="fill-foreground text-[8px]">
              {v.toFixed(2)}
            </text>
          </g>
        )
      })}
      <text x={pad.l - 4} y={H - pad.b + 4} textAnchor="end"
        className="fill-muted-foreground text-[8px]">
        0
      </text>
    </svg>
  )
}

export function PoincareViz() {
  const [result, setResult] = useState<Result | null>(null)
  const [loading, setLoading] = useState(false)

  const run = useCallback(() => {
    setLoading(true)
    void runPoincareCompositeAsync({})
      .then(setResult)
      .finally(() => setLoading(false))
  }, [])

  return (
    <Card>
      <CardHeader>
        <CardTitle>Full Poincaré composite probe</CardTitle>
        <CardDescription>
          Tests all three Poincaré generator families simultaneously on the TFIM
          lattice: rotation isotropy (J — 4-direction cardinal CV on 4×4 grid),
          multi-frame boost invariance (K — 4 spatially distinct observer clocks),
          and linear dispersion (H — ω ≈ v|k| on n=16 chain). All three must pass
          for the composite (test Z) to pass. Probe takes ~30s; dispersion uses n=16
          chain with 48 steps.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <Button onClick={run} disabled={loading} size="sm">
          {loading ? (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          ) : (
            <Play className="mr-2 h-4 w-4" />
          )}
          Run probe (~30s)
        </Button>
        {result && (
          <>
            <div className="flex flex-wrap gap-2">
              <Badge variant={result.allPoincareOk ? 'default' : 'secondary'}>
                {result.allPoincareOk
                  ? `Poincaré OK (${result.testsPassed}/${result.testsTotal})`
                  : `Partial (${result.testsPassed}/${result.testsTotal})`}
              </Badge>
              <Badge variant="outline">rot CV {result.rotationCv.toFixed(3)}</Badge>
              <Badge variant="outline">boost CV {result.boostCv.toFixed(3)}</Badge>
              <Badge variant="outline">disp R² {result.dispersionLinearR2.toFixed(3)}</Badge>
            </div>

            <div className="border rounded-md px-3 py-1">
              <PillarRow
                label="Rotation (J)"
                sublabel="4-dir cardinal CV < 0.25"
                ok={result.rotationOk}
                value={`CV=${result.rotationCv.toFixed(3)}`}
              />
              <PillarRow
                label="Boost (K) × 4 frames"
                sublabel="4-frame velocity CV < 0.25"
                ok={result.boostOk}
                value={`CV=${result.boostCv.toFixed(3)}`}
              />
              <PillarRow
                label="Dispersion (H)"
                sublabel="ω(k) linear at small k"
                ok={result.dispersionOk}
                value={`R²=${result.dispersionLinearR2.toFixed(3)}`}
              />
            </div>

            <p className="text-xs text-muted-foreground">
              Boost frames: uniform clock, corner TL, top-edge mid, corner BR — each
              measures the light-cone speed using a spatially distinct time reference
              (Page–Wootters relational boost). Low velocity CV = frame-independent.
            </p>
            <BoostFrameChart result={result} />
          </>
        )}
      </CardContent>
    </Card>
  )
}
