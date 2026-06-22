import { useMemo, useState } from 'react'
import type { RefinementDiagnostics } from '@/sim/types'

const MAX_RT_SLOPE_DEV = 0.35

function clamp01(x: number): number {
  return Math.max(0, Math.min(1, x))
}

function refinementTraceComponents(
  d: RefinementDiagnostics,
  expectedDim = 1,
) {
  return {
    rtDeficit: clamp01(1 - d.rtR2),
    slopeDeficit: clamp01(Math.abs(d.rtSlope - 1) / MAX_RT_SLOPE_DEV),
    areaPressure: d.areaPressure,
    dimPressure: clamp01(
      (d.emergentDim - expectedDim) / Math.max(1, expectedDim + 0.5),
    ),
    emergentDim: d.emergentDim,
    pressure: d.pressure,
  }
}

const SERIES = [
  { key: 'rtDeficit' as const, label: 'RT deficit (1−R²)', color: 'oklch(0.72 0.16 290)' },
  { key: 'slopeDeficit' as const, label: 'RT slope dev', color: 'oklch(0.72 0.18 25)' },
  { key: 'areaPressure' as const, label: 'Area-law proxy', color: 'oklch(0.72 0.14 155)' },
  { key: 'dimPressure' as const, label: 'Emergent dim', color: 'oklch(0.72 0.12 220)' },
  { key: 'pressure' as const, label: 'Composite pressure', color: 'oklch(0.65 0.02 290 / 0.85)', dashed: true },
]

export type DiagnosticTracePoint = {
  t: number
  step: number
  diagnostics: RefinementDiagnostics
}

function niceMax(v: number): number {
  if (v <= 0) return 1
  const pow = Math.pow(10, Math.floor(Math.log10(v)))
  const n = v / pow
  const step = n <= 1 ? 1 : n <= 2 ? 2 : n <= 5 ? 5 : 10
  return step * pow
}

export function DiagnosticTracesChart({
  slices,
  expectedDim = 1,
  triggerStep,
}: {
  slices: DiagnosticTracePoint[]
  expectedDim?: number
  /** Vertical marker when adaptive split triggers */
  triggerStep?: number
}) {
  const [visible, setVisible] = useState<Record<string, boolean>>(() =>
    Object.fromEntries(SERIES.map((s) => [s.key, true])),
  )

  const traces = useMemo(
    () =>
      slices.map((s) => ({
        t: s.t,
        step: s.step,
        ...refinementTraceComponents(s.diagnostics, expectedDim),
      })),
    [slices, expectedDim],
  )

  const W = 460
  const H = 260
  const pad = { l: 44, r: 16, t: 16, b: 52 }
  const plotW = W - pad.l - pad.r
  const plotH = H - pad.t - pad.b
  const maxT = traces[traces.length - 1]?.t ?? 1
  const maxY = niceMax(
    Math.max(
      ...traces.flatMap((d) =>
        SERIES.filter((s) => visible[s.key]).map((s) => d[s.key]),
      ),
      0.5,
    ),
  )
  const x = (t: number) => pad.l + (t / maxT) * plotW
  const y = (v: number) => pad.t + plotH - (v / maxY) * plotH
  const ticks = 4

  return (
    <div className="space-y-2">
      <svg
        viewBox={`0 0 ${W} ${H}`}
        className="w-full h-auto"
        role="img"
        aria-label="Refinement diagnostic traces over quench time"
      >
        <text
          x={pad.l - 8}
          y={pad.t + plotH / 2}
          textAnchor="middle"
          className="fill-muted-foreground text-[10px]"
          transform={`rotate(-90 ${pad.l - 8} ${pad.t + plotH / 2})`}
        >
          normalized signal
        </text>
        <text
          x={pad.l + plotW / 2}
          y={H - 8}
          textAnchor="middle"
          className="fill-muted-foreground text-[10px]"
        >
          quench time
        </text>
        {Array.from({ length: ticks + 1 }, (_, i) => {
          const v = (maxY * i) / ticks
          const yy = y(v)
          return (
            <g key={i}>
              <line
                x1={pad.l}
                y1={yy}
                x2={pad.l + plotW}
                y2={yy}
                stroke="currentColor"
                className="text-border/40"
                strokeDasharray="2 4"
              />
              <text
                x={pad.l - 6}
                y={yy + 3}
                textAnchor="end"
                className="fill-muted-foreground text-[9px]"
              >
                {v.toFixed(2)}
              </text>
            </g>
          )
        })}
        {SERIES.filter((s) => visible[s.key]).map((s) => {
          const pts = traces.map((d) => `${x(d.t)},${y(d[s.key])}`).join(' ')
          return (
            <polyline
              key={s.key}
              fill="none"
              stroke={s.color}
              strokeWidth={s.key === 'pressure' ? 1.5 : 2}
              strokeDasharray={s.dashed ? '6 4' : undefined}
              points={pts}
            />
          )
        })}
        {triggerStep != null && (() => {
          const slice = slices.find((s) => s.step === triggerStep)
          const t = slice?.t ?? triggerStep * (maxT / Math.max(slices.length - 1, 1))
          const xx = x(t)
          return (
            <line
              x1={xx}
              y1={pad.t}
              x2={xx}
              y2={pad.t + plotH}
              stroke="oklch(0.78 0.13 60)"
              strokeWidth={1.5}
              strokeDasharray="4 3"
            />
          )
        })()}
      </svg>
      <div className="flex flex-wrap gap-x-4 gap-y-1 text-xs">
        {SERIES.map((s) => (
          <label key={s.key} className="inline-flex items-center gap-1.5 cursor-pointer">
            <input
              type="checkbox"
              checked={visible[s.key]}
              onChange={() =>
                setVisible((v) => ({ ...v, [s.key]: !v[s.key] }))
              }
              className="rounded border-border"
            />
            <span
              className="inline-block w-3 h-0.5 rounded"
              style={{ background: s.color }}
            />
            <span className="text-muted-foreground">{s.label}</span>
          </label>
        ))}
      </div>
    </div>
  )
}
