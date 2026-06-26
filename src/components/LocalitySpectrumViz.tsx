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
import { runLocalitySpectrumAsync } from '@/sim/runner-async'
import type { LocalitySpectrumResult, LocalitySpectrumCase } from '@/sim/types'

type Result = LocalitySpectrumResult & { backend: string }

const GROUP_LABELS: Record<string, string> = {
  'tfim h=0.50': 'TFIM ordered (h=0.5)',
  'tfim h=1.00': 'TFIM critical (h=1.0)',
  'tfim h=1.50': 'TFIM paramagnet (h=1.5)',
}

function CaseRow({ c }: { c: LocalitySpectrumCase }) {
  return (
    <tr className="border-b border-zinc-800 last:border-0">
      <td className="py-1 pr-3 text-xs text-zinc-400">{c.model}</td>
      <td className="py-1 pr-3 text-xs text-zinc-400">seed {c.seed}</td>
      <td className="py-1 pr-3 text-xs font-mono">{c.score.toFixed(3)}</td>
      <td className="py-1 text-xs">
        <Badge variant={c.recovered ? 'default' : 'destructive'} className="text-[10px]">
          {c.recovered ? '✓ recovered' : '✗ failed'}
        </Badge>
      </td>
    </tr>
  )
}

function SummaryBar({ result }: { result: Result }) {
  const pct = Math.round(result.recoveryRate * 100)
  const W = 300
  const fill = (result.recovered / result.total) * W
  return (
    <div className="mb-4">
      <div className="flex items-center justify-between mb-1">
        <span className="text-xs text-zinc-400">
          {result.recovered}/{result.total} cases recovered ({pct}%)
        </span>
        <Badge variant={result.pass ? 'default' : 'destructive'}>
          {result.pass ? 'PASS ≥67%' : `FAIL ${pct}%`}
        </Badge>
      </div>
      <svg width={W} height={14} className="block">
        <rect x={0} y={0} width={W} height={14} rx={4} fill="#27272a" />
        <rect x={0} y={0} width={fill} height={14} rx={4} fill={result.pass ? '#22c55e' : '#ef4444'} />
        <line x1={W * 0.67} y1={0} x2={W * 0.67} y2={14} stroke="#a1a1aa" strokeWidth={1} strokeDasharray="2,2" />
      </svg>
      <div className="flex justify-between text-[10px] text-zinc-600 mt-0.5">
        <span>0%</span>
        <span style={{ marginLeft: `${W * 0.67 - 10}px` }}>67%</span>
        <span>100%</span>
      </div>
    </div>
  )
}

export function LocalitySpectrumViz() {
  const [result, setResult] = useState<Result | null>(null)
  const [running, setRunning] = useState(false)

  const run = useCallback(async () => {
    setRunning(true)
    try {
      const r = await runLocalitySpectrumAsync()
      setResult(r as Result)
    } finally {
      setRunning(false)
    }
  }, [])

  // Group cases by model+field
  const groups = result
    ? Object.entries(
        result.cases.reduce<Record<string, LocalitySpectrumCase[]>>((acc, c) => {
          const key = c.model.replace(/ seed=\d+/, '').replace(/^(\w+) n=\d+ (h=[\d.]+)$/, '$1 $2')
          acc[key] = [...(acc[key] ?? []), c]
          return acc
        }, {}),
      )
    : []

  return (
    <Card className="w-full">
      <CardHeader>
        <CardTitle>Locality from Spectrum (Test AA)</CardTitle>
        <CardDescription>
          Blind MI + bandwidth inference — no Ĥ knowledge — recovers chain factorization from eigenvectors.
          9 cases × 3 TFIM phases (ordered / critical / paramagnet), 3 seeds each.
          Pass: ≥67% correct.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <Button onClick={run} disabled={running} size="sm" className="mb-4">
          {running ? <Loader2 className="animate-spin mr-2 h-4 w-4" /> : <Play className="mr-2 h-4 w-4" />}
          Run blind inference
        </Button>

        {result && (
          <div className="space-y-4">
            <SummaryBar result={result} />

            <div className="space-y-3">
              {groups.map(([key, cases]) => {
                const label = GROUP_LABELS[key] ?? key
                const groupPass = cases.every((c) => c.recovered)
                return (
                  <div key={key}>
                    <div className="flex items-center gap-2 mb-1">
                      <span className="text-xs font-medium text-zinc-300">{label}</span>
                      <Badge
                        variant={groupPass ? 'default' : 'destructive'}
                        className="text-[10px]"
                      >
                        {cases.filter((c) => c.recovered).length}/{cases.length}
                      </Badge>
                    </div>
                    <table className="w-full">
                      <tbody>
                        {cases.map((c) => (
                          <CaseRow key={`${c.model}-${c.seed}`} c={c} />
                        ))}
                      </tbody>
                    </table>
                  </div>
                )
              })}
            </div>

            <p className="text-[11px] text-zinc-500">
              Scorer: 50% MI nearest-neighbor ratio · 35% eigenvector bandwidth · 15% emergent-dim bonus.
              Exact search over {result.cases[0]?.n ?? 6}! = {720} permutations per case.
              Backend: {result.backend} · {result.elapsedMs.toFixed(0)} ms
            </p>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
