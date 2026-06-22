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
  runBranchBornAsync,
  runEftDimensionAsync,
  runParticleStabilityAsync,
  runStabilizerSearchAsync,
  type StabilizerSearchResultWithBackend,
} from '@/sim/runner-async'
import type { BranchBornResult, EftDimensionResult, ParticleStabilityResult } from '@/sim/types'

const EXCITATION = {
  n: 8,
  field: 1.2,
  dt: 0.2,
  steps: 22,
  coupleStep: 7,
  coupling: 0.9,
  seed: 4242,
  windowRadius: 2,
}

export function MatterClassicalityViz() {
  const [stab, setStab] = useState<StabilizerSearchResultWithBackend | null>(null)
  const [particle, setParticle] = useState<ParticleStabilityResult | null>(null)
  const [born, setBorn] = useState<BranchBornResult | null>(null)
  const [eft, setEft] = useState<EftDimensionResult | null>(null)
  const [loading, setLoading] = useState(false)

  const run = useCallback(() => {
    setLoading(true)
    void Promise.all([
      runStabilizerSearchAsync(EXCITATION),
      runParticleStabilityAsync({ n: 10, dt: 0.2, steps: 24 }),
      runBranchBornAsync({ n: 8, field: 1.2, dt: 0.2, steps: 22, coupleStep: 7 }),
      runEftDimensionAsync(EXCITATION),
    ])
      .then(([s, p, b, e]) => {
        setStab(s)
        setParticle(p)
        setBorn(b)
        setEft(e)
      })
      .finally(() => setLoading(false))
  }, [])

  return (
    <Card>
      <CardHeader>
        <CardTitle>Matter &amp; classicality (S6–S8)</CardTitle>
        <CardDescription>
          Stabilizer search on branch windows, defect lifetime, Born-weight consistency, and EFT
          dimension counting.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <Button onClick={run} disabled={loading} size="sm">
          {loading ? <Loader2 className="mr-2 h-4 w-4 animate-spin" /> : <Play className="mr-2 h-4 w-4" />}
          Run Phase 6 probes
        </Button>

        {stab && (
          <div className="text-sm space-y-1">
            <div className="font-medium">I′ stabilizer search</div>
            <Badge variant={stab.stabilizer.stabilizerFound ? 'secondary' : 'destructive'}>
              {stab.stabilizer.generatorCount} generators, d={stab.stabilizer.codeDistance}
            </Badge>
            <p className="text-muted-foreground">
              {stab.stabilizer.generators.map((g) => g.label).join(', ') || 'none'}
            </p>
          </div>
        )}

        {particle && (
          <div className="text-sm space-y-1">
            <div className="font-medium">Particle stability</div>
            <Badge variant={particle.orderedLongerLived ? 'secondary' : 'outline'}>
              ordered {particle.ordered.localizationFraction.toFixed(2)} vs{' '}
              {particle.disordered.localizationFraction.toFixed(2)}
            </Badge>
          </div>
        )}

        {born && (
          <div className="text-sm space-y-1">
            <div className="font-medium">Branch Born weights</div>
            <Badge variant={born.bornConsistent ? 'secondary' : 'outline'}>
              entropy ρ={born.entropyOverlapCorr.toFixed(2)}
            </Badge>
          </div>
        )}

        {eft && (
          <div className="text-sm space-y-1">
            <div className="font-medium">EFT DOF / site</div>
            <Badge variant={eft.dofAgreement ? 'secondary' : 'outline'}>
              {eft.measuredDofPerSite.toFixed(2)} vs {eft.predictedDofPerSite.toFixed(2)} predicted
            </Badge>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
