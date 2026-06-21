import type { CouplingEdge } from '@/sim/types'

const LOCAL = 'oklch(0.72 0.16 155)'
const FAR = 'oklch(0.72 0.14 25)'

export function CouplingGraph({
  title,
  n,
  edges,
}: {
  title: string
  n: number
  edges: CouplingEdge[]
}) {
  const W = 360
  const H = 100
  const pad = 24
  const span = W - 2 * pad
  const y = H / 2
  const x = (i: number) => pad + (i / Math.max(n - 1, 1)) * span

  return (
    <div className="space-y-2">
      <p className="text-sm font-medium">{title}</p>
      <svg viewBox={`0 0 ${W} ${H}`} className="w-full h-auto" role="img" aria-label={title}>
        <line x1={pad} y1={y} x2={W - pad} y2={y} stroke="currentColor" className="text-border" />
        {Array.from({ length: n }, (_, i) => (
          <circle key={i} cx={x(i)} cy={y} r={5} fill="currentColor" className="text-primary" />
        ))}
        {edges.map((e, k) => {
          const local = e.dist === 1
          return (
            <line
              key={k}
              x1={x(e.i)}
              y1={y}
              x2={x(e.j)}
              y2={y}
              stroke={local ? LOCAL : FAR}
              strokeWidth={local ? 2 : 1}
              strokeOpacity={local ? 0.9 : 0.35}
            />
          )
        })}
      </svg>
      <p className="text-xs text-muted-foreground">
        Green = nearest-neighbour; warm = longer-range two-body terms.
      </p>
    </div>
  )
}
