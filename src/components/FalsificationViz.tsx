import { Link } from 'react-router-dom'
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
  runFalsificationBatteryAsync,
  type FalsificationBatteryResultWithBackend,
} from '@/sim/runner-async'

export function FalsificationViz() {
  const [result, setResult] = useState<FalsificationBatteryResultWithBackend | null>(
    null,
  )
  const [loading, setLoading] = useState(false)

  const run = useCallback(() => {
    setLoading(true)
    void runFalsificationBatteryAsync()
      .then(setResult)
      .finally(() => setLoading(false))
  }, [])

  return (
    <Card>
      <CardHeader>
        <CardTitle>Falsification battery</CardTitle>
        <CardDescription>
          Tests that would disprove Mad-Dog-specific claims — not generic gapped-
          system properties.{' '}
          <Link to="/falsification" className="underline hover:text-foreground">
            How the battery works
          </Link>
          .
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <Button onClick={run} disabled={loading} size="sm">
          {loading ? (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          ) : (
            <Play className="mr-2 h-4 w-4" />
          )}
          Run falsification battery
        </Button>
        {result && (
          <>
            <div className="flex flex-wrap gap-2 text-xs">
              <Badge variant={result.passed === result.total ? 'secondary' : 'destructive'}>
                {result.passed}/{result.total} passed
              </Badge>
              <Badge variant="outline">
                {result.elapsedMs.toFixed(0)} ms · {result.backend}
              </Badge>
            </div>
            <ul className="space-y-2 text-sm">
              {result.tests.map((t) => (
                <li key={t.id} className="rounded-md border p-3">
                  <div className="flex items-center gap-2">
                    <Badge variant={t.passed ? 'secondary' : 'destructive'}>
                      {t.passed ? 'PASS' : 'FAIL'}
                    </Badge>
                    <span className="font-medium">
                      [{t.id}] {t.name}
                    </span>
                  </div>
                  <p className="mt-1 text-xs text-muted-foreground">{t.detail}</p>
                </li>
              ))}
            </ul>
          </>
        )}
      </CardContent>
    </Card>
  )
}
