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
  runLorentzScalingAsync,
  runDispersionAsync,
  runBoostInvarianceAsync,
} from '@/sim/runner-async'
import type {
  LorentzScalingResult,
  DispersionResult,
  BoostInvarianceResult,
} from '@/sim/types'

export function CausalityViz() {
  const [scaling, setScaling] = useState<LorentzScalingResult | null>(null)
  const [dispersion, setDispersion] = useState<DispersionResult | null>(null)
  const [boost, setBoost] = useState<BoostInvarianceResult | null>(null)
  const [loading, setLoading] = useState(false)

  const run = useCallback(() => {
    setLoading(true)
    Promise.all([
      runLorentzScalingAsync(),
      runDispersionAsync({ n: 16, field: 1.0, dt: 0.15, steps: 48, modes: 3 }),
      runBoostInvarianceAsync({ rows: 4, cols: 4, field: 1.2, dt: 0.2, steps: 32, edgeSite: 0 }),
    ])
      .then(([s, d, b]) => {
        setScaling(s)
        setDispersion(d)
        setBoost(b)
      })
      .finally(() => setLoading(false))
  }, [])

  return (
    <Card>
      <CardHeader>
        <CardTitle>Causality → Lorentz (Phase 4)</CardTitle>
        <CardDescription>
          Cardinal speed isotropy scaling on grids, linear ω(k) from wavepacket
          propagation, and light-cone speed invariance under clock-subset
          observers.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <Button onClick={run} disabled={loading} size="sm">
          {loading ? (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          ) : (
            <Play className="mr-2 h-4 w-4" />
          )}
          Run causality suite
        </Button>

        {scaling && (
          <div className="space-y-2 text-sm">
            <h4 className="font-medium">Lorentz scaling (L′)</h4>
            <div className="flex flex-wrap gap-2 text-xs">
              {scaling.cases.map((c) => (
                <Badge key={c.label} variant={c.passed ? 'secondary' : 'outline'}>
                  {c.label}: CoV={c.speedCv.toFixed(3)}
                </Badge>
              ))}
              <Badge variant={scaling.covImproves ? 'secondary' : 'outline'}>
                improves={scaling.covImproves ? 'yes' : 'no'}
              </Badge>
            </div>
          </div>
        )}

        {dispersion && (
          <div className="space-y-2 text-sm">
            <h4 className="font-medium">Dispersion (O)</h4>
            <p className="text-xs text-muted-foreground font-mono">
              ω slope={dispersion.omegaSlope.toFixed(3)} R²={dispersion.linearR2.toFixed(3)}{' '}
              m_eff≈{dispersion.effectiveMass.toFixed(3)} linear=
              {dispersion.linearAtSmallK ? 'yes' : 'no'}
            </p>
          </div>
        )}

        {boost && (
          <div className="space-y-2 text-sm">
            <h4 className="font-medium">Boost invariance (P)</h4>
            <p className="text-xs text-muted-foreground font-mono">
              v_uniform={boost.velocityUniform.toFixed(3)} v_edge=
              {boost.velocityEdgeClock.toFixed(3)} Δ=
              {boost.relativeDelta.toFixed(3)}
            </p>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
