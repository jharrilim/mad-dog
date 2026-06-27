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
import { runFactorizationCompareAsync } from '@/sim/runner-async'
import type { FactorizationCompareResult } from '@/sim/types'

const COMPARE_CFG = {
  nSmall: 8,
  field: 1.5,
  dt: 0.2,
  step: 12,
  seed: 4242,
  deltaN: 2,
}

export function FactorizationCompareViz() {
  const [result, setResult] = useState<FactorizationCompareResult | null>(null)
  const [loading, setLoading] = useState(false)

  const run = useCallback(() => {
    setLoading(true)
    void runFactorizationCompareAsync(COMPARE_CFG)
      .then(setResult)
      .finally(() => setLoading(false))
  }, [])

  return (
    <Card>
      <CardHeader>
        <CardTitle>Same |ψ⟩, different n</CardTitle>
        <CardDescription>
          Snapshot a quenched state on n qubits, embed into n+Δ with |0⟩ padding, truncate back,
          and compare blind factorization locality, emergent dimension, and refinement pressure.
          Control: native quench on n+Δ (different state).
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <Button onClick={run} disabled={loading} size="sm">
          {loading ? (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          ) : (
            <Play className="mr-2 h-4 w-4" />
          )}
          Run embed / truncate compare
        </Button>

        {result && (
          <div className="text-sm space-y-3">
            <div className="flex flex-wrap gap-2 items-center">
              <Badge variant={result.embeddingFaithful ? 'secondary' : 'destructive'}>
                AG faithful={result.embeddingFaithful ? 'yes' : 'no'}
              </Badge>
              <span className="text-muted-foreground">
                n={result.nSmall}→{result.nLarge} @ site {result.splitSite}, step {result.step}
              </span>
            </div>
            <p className="text-muted-foreground">
              roundtrip F={result.roundtripFidelity.toFixed(4)} · Δloc_emb=
              {result.localityDriftEmbed.toFixed(3)} · Δloc_rt=
              {result.localityDriftRoundtrip.toFixed(3)} · Δdim_rt={result.dimDriftRoundtrip}
            </p>
            <div className="overflow-x-auto">
              <table className="w-full text-left text-xs border-collapse">
                <thead>
                  <tr className="border-b">
                    <th className="py-1 pr-3 font-medium">case</th>
                    <th className="py-1 pr-3 font-medium">n</th>
                    <th className="py-1 pr-3 font-medium">locality</th>
                    <th className="py-1 pr-3 font-medium">dim</th>
                    <th className="py-1 font-medium">pressure</th>
                  </tr>
                </thead>
                <tbody>
                  {result.cases.map((c) => (
                    <tr key={c.label} className="border-b border-border/50">
                      <td className="py-1 pr-3 font-mono">{c.label}</td>
                      <td className="py-1 pr-3">{c.n}</td>
                      <td className="py-1 pr-3">{c.localityFraction.toFixed(3)}</td>
                      <td className="py-1 pr-3">{c.emergentDim}</td>
                      <td className="py-1">{c.pressure.toFixed(3)}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
