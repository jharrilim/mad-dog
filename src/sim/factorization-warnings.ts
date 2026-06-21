/**
 * Cost estimates and runtime warnings for factorization search (CPU-heavy).
 */

import type {
  FactorizationInputMode,
  FactorizationRefinementConfig,
  FactorizationSearchConfig,
  FactorizationSearchMethod,
} from './types.ts'

export type FactorizationCostSeverity = 'info' | 'warn' | 'critical'

export interface FactorizationCostWarning {
  severity: FactorizationCostSeverity
  /** Short line for UI badges. */
  summary: string
  /** Longer guidance (console / banner body). */
  detail: string
  /** Rough count of permutation scoring passes (dominant cost). */
  estimatedScoreEvaluations: number
}

function factorial(n: number): number {
  let out = 1
  for (let i = 2; i <= n; i++) out *= i
  return out
}

function resolveSearchMethod(
  method: FactorizationSearchMethod | undefined,
  n: number,
): FactorizationSearchMethod {
  if (method === 'exact' || method === 'greedy' || method === 'annealing') {
    return method
  }
  return n <= 8 ? 'exact' : 'annealing'
}

function severityFromEvals(
  evals: number,
  extras: { exactN?: number; eigenstateCount?: number },
): FactorizationCostSeverity {
  const { exactN, eigenstateCount = 1 } = extras
  if (exactN !== undefined && exactN >= 7) return 'critical'
  if (evals >= 50_000 || (evals >= 10_000 && eigenstateCount > 1)) {
    return 'critical'
  }
  if (evals >= 5_000) return 'warn'
  return 'info'
}

/** Estimate cost of a single factorization search run. */
export function estimateFactorizationSearchCost(
  config: FactorizationSearchConfig,
): FactorizationCostWarning {
  const n = config.n
  const method = resolveSearchMethod(config.searchMethod, n)
  const annealingSteps = Math.max(100, config.annealingSteps ?? 3000)
  const eigenstateCount = Math.min(
    4,
    Math.max(1, config.eigenstateCount ?? 1),
  )
  const inputMode: FactorizationInputMode = config.inputMode ?? 'pauli'

  let estimatedScoreEvaluations: number
  let methodLabel: string

  if (method === 'exact' && n <= 8) {
    estimatedScoreEvaluations = factorial(n)
    methodLabel = `exact enumeration (${estimatedScoreEvaluations.toLocaleString()} permutations)`
  } else if (method === 'greedy') {
    estimatedScoreEvaluations = 40
    methodLabel = 'greedy local search'
  } else {
    estimatedScoreEvaluations = annealingSteps + 17
    methodLabel = `simulated annealing (${annealingSteps.toLocaleString()} steps)`
  }

  if (method === 'exact' && n > 8) {
    methodLabel += ' (requested exact, but n>8 falls back to annealing in engine)'
  }

  const severity = severityFromEvals(estimatedScoreEvaluations, {
    exactN: method === 'exact' && n <= 8 ? n : undefined,
    eigenstateCount,
  })
  let resolvedSeverity = severity
  if (
    method === 'annealing' &&
    ((n >= 9 && annealingSteps >= 1000) || (n >= 10 && annealingSteps >= 500))
  ) {
    resolvedSeverity =
      resolvedSeverity === 'info' ? 'warn' : resolvedSeverity
  }
  if (inputMode === 'spectrum' && n >= 8 && resolvedSeverity === 'info') {
    resolvedSeverity = 'warn'
  }

  const eigenNote =
    eigenstateCount > 1
      ? ` Scores ${eigenstateCount} low-energy states (extra MI work).`
      : ''
  const spectrumNote =
    inputMode === 'spectrum'
      ? ' Spectrum mode diagonalizes the full Hamiltonian first.'
      : ''

  const timeHint =
    resolvedSeverity === 'critical'
      ? 'This can take 30+ minutes on WASM.'
      : resolvedSeverity === 'warn'
        ? 'This may take several minutes; keep the tab open.'
        : 'Should finish quickly on WASM.'

  return {
    severity: resolvedSeverity,
    summary:
      resolvedSeverity === 'critical'
        ? 'Very expensive factorization search'
        : resolvedSeverity === 'warn'
          ? 'Slow factorization search'
          : 'Factorization search',
    detail: `${methodLabel} on n=${n}.${eigenNote}${spectrumNote} ${timeHint}`,
    estimatedScoreEvaluations,
  }
}

/** Estimate cost of the joint quench + factorization drift study. */
export function estimateFactorizationRefinementCost(
  config: FactorizationRefinementConfig,
): FactorizationCostWarning {
  const steps = config.steps
  const annealingSteps = config.annealingSteps ?? 800
  const slices = steps + 1
  const estimatedScoreEvaluations = slices * annealingSteps

  const severity: FactorizationCostSeverity =
    estimatedScoreEvaluations >= 8_000
      ? 'critical'
      : estimatedScoreEvaluations >= 4_000
        ? 'warn'
        : 'info'

  const timeHint =
    severity === 'critical'
      ? 'Often takes 30+ minutes — each quench slice re-runs annealing search.'
      : severity === 'warn'
        ? 'May take several minutes.'
        : 'Moderate runtime expected.'

  return {
    severity,
    summary:
      severity === 'critical'
        ? 'Very expensive quench + factorization study'
        : severity === 'warn'
          ? 'Slow quench + factorization study'
          : 'Quench + factorization study',
    detail: `${slices} time slices × ${annealingSteps.toLocaleString()} annealing steps ≈ ${estimatedScoreEvaluations.toLocaleString()} scoring passes on n=${config.n}. ${timeHint} Prefer WASM; reduce steps or annealingSteps to speed up.`,
    estimatedScoreEvaluations,
  }
}

const LOG_PREFIX = '[mad-dog factorization]'

/** Log to console when severity is warn or critical. Returns the estimate. */
export function warnIfExpensiveFactorizationSearch(
  config: FactorizationSearchConfig,
  context?: string,
): FactorizationCostWarning {
  const warning = estimateFactorizationSearchCost(config)
  if (warning.severity === 'warn' || warning.severity === 'critical') {
    const ctx = context ? ` (${context})` : ''
    console.warn(`${LOG_PREFIX}${ctx} ${warning.summary}: ${warning.detail}`)
  }
  return warning
}

export function warnIfExpensiveFactorizationRefinement(
  config: FactorizationRefinementConfig,
  context?: string,
): FactorizationCostWarning {
  const warning = estimateFactorizationRefinementCost(config)
  if (warning.severity === 'warn' || warning.severity === 'critical') {
    const ctx = context ? ` (${context})` : ''
    console.warn(`${LOG_PREFIX}${ctx} ${warning.summary}: ${warning.detail}`)
  }
  return warning
}
