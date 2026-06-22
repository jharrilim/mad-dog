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
  runModularDualClockAsync,
  type ModularDualClockResultWithBackend,
} from '@/sim/runner-async'

function TauChart({ result }: { result: ModularDualClockResultWithBackend }) {
  const W = 460
  const H = 240
  const pad = { l: 44, r: 16, t: 16, b: 38 }
  const plotW = W - pad.l - pad.r
  const plotH = H - pad.t - pad.b
  const n = result.modularTauA.length
  const maxTau = Math.max(
    result.modularTauA[n - 1] ?? 1,
    result.modularTauB[n - 1] ?? 1,
    1e-6,
  )
  const x = (k: number) => pad.l + (k / Math.max(n - 1, 1)) * plotW
  const y = (v: number) => pad.t + plotH - (v / maxTau) * plotH
  const lineA = result.modularTauA.map((v, k) => `${x(k)},${y(v)}`).join(' ')
  const lineB = result.modularTauB.map((v, k) => `${x(k)},${y(v)}`).join(' ')

  return (
    <svg viewBox={`0 0 ${W} ${H}`} className="w-full h-auto" role="img">
      <polyline
        fill="none"
        stroke="oklch(0.72 0.16 290)"
        strokeWidth={2}
        points={lineA}
      />
      <polyline
        fill="none"
        stroke="oklch(0.72 0.14 155)"
        strokeWidth={2}
        strokeDasharray="5 3"
        points={lineB}
      />
      <text x={pad.l + 4} y={pad.t + 12} className="fill-muted-foreground text-[9px]">
        solid = region A ({result.regionA.join(',')})
      </text>
      <text x={pad.l + 4} y={pad.t + 24} className="fill-muted-foreground text-[9px]">
        dashed = region B ({result.regionB.join(',')})
      </text>
    </svg>
  )
}

export function ModularClockViz() {
  const [result, setResult] =
    useState<ModularDualClockResultWithBackend | null>(null)
  const [loading, setLoading] = useState(false)

  const run = useCallback(() => {
    setLoading(true)
    void runModularDualClockAsync({
      n: 12,
      field: 1.2,
      dt: 0.15,
      steps: 48,
      modularSlices: 12,
    })
      .then(setResult)
      .finally(() => setLoading(false))
  }, [])

  return (
    <Card>
      <CardHeader>
        <CardTitle>Modular dual clocks (in-cone)</CardTitle>
        <CardDescription>
          Two small regions inside the light cone, both ticked by entanglement-
          spectrum drift (discrete K_A = −log ρ_A proxy) rather than Z-threshold
          crossings. Desynchronization here is entanglement-structure-driven, not
          Lieb–Robinson arrival delay.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <Button onClick={run} disabled={loading} size="sm">
          {loading ? (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          ) : (
            <Play className="mr-2 h-4 w-4" />
          )}
          Run modular clocks
        </Button>

        {result && (
          <>
            <div className="flex flex-wrap gap-2 text-xs">
              <Badge variant="outline">
                sync R² modular = {result.syncR2Modular.toFixed(3)}
              </Badge>
              <Badge variant="outline">
                sync R² Z-threshold = {result.syncR2Z.toFixed(3)}
              </Badge>
              <Badge variant="outline">
                min mod ↔ uniform = {result.minModularUniformR2.toFixed(3)}
              </Badge>
              <Badge variant="outline">
                min Z edge ↔ uniform = {result.minZEdgeUniformR2.toFixed(3)}
              </Badge>
              <Badge variant="outline">
                {result.elapsedMs.toFixed(0)} ms · {result.backend}
              </Badge>
            </div>
            <TauChart result={result} />
            <p className="text-xs text-muted-foreground">
              Cumulative modular time τ vs uniform Schrödinger step. Not full
              Tomita–Takesaki modular flow — a finite-dimensional proxy.
            </p>
          </>
        )}
      </CardContent>
    </Card>
  )
}
