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
  runFactorizationSearchAsync,
  type FactorizationSearchResultWithBackend,
} from '@/sim/runner-async'
import type { FactorizationKind } from '@/sim/factorization'

function permLabel(perm: number[]): string {
  return perm.map((p) => String(p)).join('→')
}

export function FactorizationViz() {
  const [kind, setKind] = useState<FactorizationKind>('shuffled_chain')
  const [result, setResult] = useState<FactorizationSearchResultWithBackend | null>(
    null,
  )
  const [loading, setLoading] = useState(false)

  const run = useCallback(() => {
    setLoading(true)
    void runFactorizationSearchAsync({
      kind,
      n: 6,
      field: 1.5,
      seed: 4242,
      topK: 5,
    })
      .then(setResult)
      .finally(() => setLoading(false))
  }, [kind])

  return (
    <Card>
      <CardHeader>
        <CardTitle>Locality from the spectrum (prototype)</CardTitle>
        <CardDescription>
          Given a Hamiltonian and its ground state, search qubit label permutations
          that make coupling look nearest-neighbour on a line. Score blends H
          locality with MI dominance between line neighbours.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="flex flex-wrap gap-2">
          <Button
            size="sm"
            variant={kind === 'shuffled_chain' ? 'default' : 'outline'}
            onClick={() => setKind('shuffled_chain')}
          >
            Shuffled chain
          </Button>
          <Button
            size="sm"
            variant={kind === 'random' ? 'default' : 'outline'}
            onClick={() => setKind('random')}
          >
            Random non-local
          </Button>
        </div>

        <Button onClick={run} disabled={loading} size="sm">
          {loading ? (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          ) : (
            <Play className="mr-2 h-4 w-4" />
          )}
          Run factorization search
        </Button>

        {result && (
          <>
            <div className="flex flex-wrap gap-2 text-xs">
              <Badge variant="outline">{result.label}</Badge>
              <Badge variant="outline">
                {result.elapsedMs.toFixed(0)} ms · {result.backend}
              </Badge>
              <Badge variant={result.recoveredIdentity ? 'secondary' : 'destructive'}>
                {result.recoveredIdentity ? 'local factorization found' : 'weak recovery'}
              </Badge>
            </div>

            <div className="grid sm:grid-cols-2 gap-3 text-sm">
              <div className="rounded-md border p-3 space-y-2">
                <p className="font-medium">Baseline (identity labeling)</p>
                <dl className="grid grid-cols-2 gap-x-3 gap-y-1 text-muted-foreground">
                  <dt>locality</dt>
                  <dd className="text-foreground tabular-nums">
                    {(result.baseline.localityFraction * 100).toFixed(0)}%
                  </dd>
                  <dt>MI nn ratio</dt>
                  <dd className="text-foreground tabular-nums">
                    {result.baseline.miNnRatio.toFixed(2)}
                  </dd>
                  <dt>score</dt>
                  <dd className="text-foreground tabular-nums">
                    {result.baseline.score.toFixed(3)}
                  </dd>
                  <dt>non-local terms</dt>
                  <dd className="text-foreground tabular-nums">
                    {result.baseline.nonlocalTerms}
                  </dd>
                </dl>
              </div>
              <div className="rounded-md border p-3 space-y-2">
                <p className="font-medium">Best permutation</p>
                <p className="font-mono text-xs text-muted-foreground break-all">
                  {permLabel(result.best.permutation)}
                </p>
                <dl className="grid grid-cols-2 gap-x-3 gap-y-1 text-muted-foreground">
                  <dt>locality</dt>
                  <dd className="text-foreground tabular-nums">
                    {(result.best.localityFraction * 100).toFixed(0)}%
                  </dd>
                  <dt>MI nn ratio</dt>
                  <dd className="text-foreground tabular-nums">
                    {result.best.miNnRatio.toFixed(2)}
                  </dd>
                  <dt>score</dt>
                  <dd className="text-foreground tabular-nums">
                    {result.best.score.toFixed(3)}
                  </dd>
                  <dt>non-local terms</dt>
                  <dd className="text-foreground tabular-nums">
                    {result.best.nonlocalTerms}
                  </dd>
                </dl>
              </div>
            </div>

            {result.topCandidates.length > 1 && (
              <div className="text-xs text-muted-foreground space-y-1">
                <p className="font-medium text-foreground text-sm">Top candidates</p>
                {result.topCandidates.map((c, i) => (
                  <p key={i} className="font-mono">
                    #{i + 1} score={c.score.toFixed(3)} locality=
                    {(c.localityFraction * 100).toFixed(0)}% perm=
                    {permLabel(c.permutation)}
                  </p>
                ))}
              </div>
            )}

            <p className="text-sm text-muted-foreground leading-relaxed">
              {kind === 'shuffled_chain'
                ? 'A local TFIM chain with randomly permuted qubit labels. A successful search recovers a line factorization (locality ≈ 100%).'
                : 'A fully non-local random Hamiltonian — the best line factorization may still score poorly; this probes how much locality the spectrum admits.'}
            </p>
          </>
        )}
      </CardContent>
    </Card>
  )
}
