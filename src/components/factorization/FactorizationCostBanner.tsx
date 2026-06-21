import type { FactorizationCostWarning } from '@/sim/factorization-warnings'

export function FactorizationCostBanner({
  warning,
}: {
  warning: FactorizationCostWarning
}) {
  if (warning.severity === 'info') return null

  const styles =
    warning.severity === 'critical'
      ? 'border-destructive/50 bg-destructive/10 text-destructive'
      : 'border-amber-500/50 bg-amber-500/10 text-amber-800 dark:text-amber-300'

  return (
    <div
      className={`rounded-md border px-3 py-2 text-sm ${styles}`}
      role="alert"
    >
      <p className="font-medium">{warning.summary}</p>
      <p className="mt-1 text-xs opacity-90">{warning.detail}</p>
    </div>
  )
}
