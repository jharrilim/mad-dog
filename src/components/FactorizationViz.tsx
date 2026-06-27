import { useCallback, useMemo, useState } from 'react'
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
import { MiHeatmap } from '@/components/factorization/MiHeatmap'
import { CouplingGraph } from '@/components/factorization/CouplingGraph'
import {
  runFactorizationSearchAsync,
  type FactorizationSearchResultWithBackend,
} from '@/sim/runner-async'
import { FactorizationCostBanner } from '@/components/factorization/FactorizationCostBanner'
import { estimateFactorizationSearchCost } from '@/sim/factorization-warnings'
import type {
  FactorizationInputMode,
  FactorizationKind,
  FactorizationSearchConfig,
} from '@/sim/types'

function permLabel(perm: number[]): string {
  return perm.map((p) => String(p)).join('→')
}

export function FactorizationViz() {
  const [kind, setKind] = useState<FactorizationKind>('shuffled_chain')
  const [inputMode, setInputMode] = useState<FactorizationInputMode>('pauli')
  const [result, setResult] = useState<FactorizationSearchResultWithBackend | null>(
    null,
  )
  const [loading, setLoading] = useState(false)

  const searchConfig = useMemo((): FactorizationSearchConfig => {
    const lattice =
      kind === 'shuffled_grid'
        ? { n: 9, rows: 3, cols: 3, graphKind: 'grid' as const }
        : kind === 'shuffled_torus'
          ? { n: 4, rows: 2, cols: 2, graphKind: 'torus' as const }
          : kind === 'shuffled_cube'
            ? { n: 8, lx: 2, ly: 2, lz: 2, graphKind: 'cube' as const }
            : { n: 6, graphKind: 'line' as const }
    return {
      kind,
      field: 1.5,
      seed: 4242,
      topK: 5,
      inputMode,
      eigenstateCount: inputMode === 'spectrum' ? 3 : 1,
      searchMethod:
        kind === 'random' || kind === 'shuffled_grid' ? undefined : 'exact',
      ...lattice,
    }
  }, [kind, inputMode])

  const costWarning = useMemo(
    () => estimateFactorizationSearchCost(searchConfig),
    [searchConfig],
  )

  const run = useCallback(() => {
    setLoading(true)
    void runFactorizationSearchAsync(searchConfig)
      .then(setResult)
      .finally(() => setLoading(false))
  }, [searchConfig])

  return (
    <Card>
      <CardHeader>
        <CardTitle>Locality from H and low-energy states</CardTitle>
        <CardDescription>
          Search qubit label permutations that make coupling look local on a line,
          grid, torus, or cube. <strong>Pauli mode</strong> uses Ĥ structure + MI;{' '}
          <strong>spectrum mode</strong> scores from eigenvectors only (no Pauli
          terms in the search).
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="flex flex-wrap gap-2">
          <Button
            size="sm"
            variant={kind === 'shuffled_chain' ? 'default' : 'outline'}
            onClick={() => setKind('shuffled_chain')}
          >
            TFIM chain
          </Button>
          <Button
            size="sm"
            variant={kind === 'shuffled_xx_chain' ? 'default' : 'outline'}
            onClick={() => setKind('shuffled_xx_chain')}
          >
            XX chain
          </Button>
          <Button
            size="sm"
            variant={kind === 'shuffled_heisenberg_chain' ? 'default' : 'outline'}
            onClick={() => setKind('shuffled_heisenberg_chain')}
          >
            Heisenberg
          </Button>
          <Button
            size="sm"
            variant={kind === 'shuffled_sparse_chain' ? 'default' : 'outline'}
            onClick={() => setKind('shuffled_sparse_chain')}
          >
            Sparse local
          </Button>
          <Button
            size="sm"
            variant={kind === 'shuffled_grid' ? 'default' : 'outline'}
            onClick={() => setKind('shuffled_grid')}
          >
            Shuffled grid
          </Button>
          <Button
            size="sm"
            variant={kind === 'shuffled_torus' ? 'default' : 'outline'}
            onClick={() => setKind('shuffled_torus')}
          >
            Shuffled torus
          </Button>
          <Button
            size="sm"
            variant={kind === 'shuffled_cube' ? 'default' : 'outline'}
            onClick={() => setKind('shuffled_cube')}
          >
            Shuffled cube
          </Button>
          <Button
            size="sm"
            variant={kind === 'random' ? 'default' : 'outline'}
            onClick={() => setKind('random')}
          >
            Random non-local
          </Button>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button
            size="sm"
            variant={inputMode === 'pauli' ? 'default' : 'outline'}
            onClick={() => setInputMode('pauli')}
          >
            Pauli + MI
          </Button>
          <Button
            size="sm"
            variant={inputMode === 'spectrum' ? 'default' : 'outline'}
            onClick={() => setInputMode('spectrum')}
          >
            Spectrum only
          </Button>
        </div>

        <FactorizationCostBanner warning={costWarning} />

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
              <Badge variant="outline">{result.scorerUsed}</Badge>
              <Badge variant="outline">
                {result.searchMethod} ({result.searchIters} iters)
              </Badge>
              <Badge variant={result.recoveredIdentity ? 'secondary' : 'destructive'}>
                {result.recoveredIdentity ? 'local factorization found' : 'weak recovery'}
              </Badge>
              {result.permMatchDistance !== undefined && (
                <Badge variant="outline">perm match {result.permMatchDistance}</Badge>
              )}
            </div>

            <div className="grid sm:grid-cols-2 gap-3 text-sm">
              <div className="rounded-md border p-3 space-y-2">
                <p className="font-medium">Baseline (identity labeling)</p>
                <dl className="grid grid-cols-2 gap-x-3 gap-y-1 text-muted-foreground">
                  <dt>locality</dt>
                  <dd className="text-foreground tabular-nums">
                    {(result.baseline.localityFraction * 100).toFixed(0)}%
                  </dd>
                  <dt>emergent dim</dt>
                  <dd className="text-foreground tabular-nums">{result.baseline.emergentDim}</dd>
                  <dt>score</dt>
                  <dd className="text-foreground tabular-nums">{result.baseline.score.toFixed(3)}</dd>
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
                  <dt>emergent dim</dt>
                  <dd className="text-foreground tabular-nums">{result.best.emergentDim}</dd>
                  <dt>score</dt>
                  <dd className="text-foreground tabular-nums">{result.best.score.toFixed(3)}</dd>
                </dl>
              </div>
            </div>

            {result.uniqueness && (
              <div className="rounded-md border p-3 text-sm space-y-1">
                <p className="font-medium">Uniqueness (top-k equivalence classes)</p>
                <dl className="grid grid-cols-2 gap-x-3 gap-y-1 text-muted-foreground">
                  <dt>classes</dt>
                  <dd className="text-foreground tabular-nums">
                    {result.uniqueness.equivalenceClassCount}
                  </dd>
                  <dt>best class size</dt>
                  <dd className="text-foreground tabular-nums">
                    {result.uniqueness.bestClassSize}
                  </dd>
                  <dt>true in top-k</dt>
                  <dd className="text-foreground">{result.uniqueness.trueInTopK ? 'yes' : 'no'}</dd>
                  {result.uniqueness.trueClassRank !== undefined && (
                    <>
                      <dt>true class rank</dt>
                      <dd className="text-foreground tabular-nums">
                        {result.uniqueness.trueClassRank}
                      </dd>
                    </>
                  )}
                  <dt>score gap</dt>
                  <dd className="text-foreground tabular-nums">
                    {result.uniqueness.scoreGapToSecondClass.toFixed(3)}
                  </dd>
                </dl>
              </div>
            )}

            <div className="grid sm:grid-cols-2 gap-6">
              <MiHeatmap title="MI — baseline labeling" mi={result.baselineMi} />
              <MiHeatmap title="MI — best permutation" mi={result.bestMi} />
            </div>

            {result.couplingEdges.length > 0 && (
              <CouplingGraph
                title="Two-body couplings (best labeling)"
                n={result.n}
                edges={result.couplingEdges}
              />
            )}

            <p className="text-sm text-muted-foreground leading-relaxed">
              {kind === 'random'
                ? 'A fully non-local Hamiltonian — the best line factorization may still score poorly.'
                : 'A local model with hidden qubit labels. Successful search recovers high locality on the target graph.'}
            </p>
          </>
        )}
      </CardContent>
    </Card>
  )
}
