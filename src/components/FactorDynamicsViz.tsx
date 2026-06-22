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
import { runHolographicBoundAsync, runInplaceSplitAsync } from '@/sim/runner-async'
import type { HolographicBoundResult, InplaceSplitResult } from '@/sim/types'

const SPLIT_CFG = {
  n: 10,
  field: 1.5,
  dt: 0.2,
  steps: 18,
  seed: 7711,
  deltaN: 2,
}

const BOUND_CFG = { field: 1.5, nMin: 6, nMax: 12 }

export function FactorDynamicsViz() {
  const [split, setSplit] = useState<InplaceSplitResult | null>(null)
  const [bound, setBound] = useState<HolographicBoundResult | null>(null)
  const [loading, setLoading] = useState(false)

  const run = useCallback(() => {
    setLoading(true)
    void Promise.all([runInplaceSplitAsync(SPLIT_CFG), runHolographicBoundAsync(BOUND_CFG)])
      .then(([s, b]) => {
        setSplit(s)
        setBound(b)
      })
      .finally(() => setLoading(false))
  }, [])

  const ev = split?.splitEvent

  return (
    <Card>
      <CardHeader>
        <CardTitle>Factor count dynamics (S10)</CardTitle>
        <CardDescription>
          In-place tensor split on the same |ψ⟩ vs full n+Δ re-quench, and minimal n where area law,
          emergent dimension, and Hamiltonian locality co-satisfy.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <Button onClick={run} disabled={loading} size="sm">
          {loading ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <Play className="mr-2 h-4 w-4" />}
          Run Phase 7 probes
        </Button>

        {ev && (
          <div className="text-sm space-y-1">
            <div className="font-medium">V′ in-place split</div>
            <Badge variant={ev.inPlaceImproves ? 'secondary' : 'destructive'}>
              step {ev.triggerStep} @ site {ev.splitSite}
            </Badge>
            <p className="text-muted-foreground">
              pressure {ev.pre.pressure.toFixed(3)} → {ev.inPlace.pressure.toFixed(3)} (Δ{' '}
              {ev.pressureDelta.toFixed(3)}); rerun {ev.rerun.pressure.toFixed(3)}
            </p>
          </div>
        )}

        {bound && (
          <div className="text-sm space-y-1">
            <div className="font-medium">W′ holographic bound</div>
            <Badge variant={bound.nMin != null && bound.boundScales ? 'secondary' : 'outline'}>
              n_min={bound.nMin ?? '—'} saturation={bound.nSaturation}
            </Badge>
            <p className="text-muted-foreground">
              {bound.points
                .map((p) => `${p.n}:${p.allOk ? '✓' : '·'}`)
                .join(' ')}
            </p>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
