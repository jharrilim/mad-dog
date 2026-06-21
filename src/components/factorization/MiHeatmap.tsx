const COLD = 'oklch(0.25 0.02 290)'
const HOT = 'oklch(0.78 0.18 290)'

function lerpColor(t: number): string {
  const clamped = Math.max(0, Math.min(1, t))
  return `color-mix(in oklch, ${COLD} ${(1 - clamped) * 100}%, ${HOT})`
}

export function MiHeatmap({
  title,
  mi,
}: {
  title: string
  mi: number[][]
}) {
  const n = mi.length
  const max = Math.max(...mi.flatMap((row) => row), 1e-9)
  const cell = 22
  const pad = 28
  const W = pad + n * cell + 8
  const H = pad + n * cell + 24

  return (
    <div className="space-y-2">
      <p className="text-sm font-medium">{title}</p>
      <svg viewBox={`0 0 ${W} ${H}`} className="w-full max-w-xs h-auto" role="img" aria-label={title}>
        {mi.map((row, i) =>
          row.map((v, j) => (
            <rect
              key={`${i}-${j}`}
              x={pad + j * cell}
              y={pad + i * cell}
              width={cell - 1}
              height={cell - 1}
              rx={2}
              fill={lerpColor(v / max)}
            />
          )),
        )}
        <text x={pad} y={14} className="fill-muted-foreground text-[9px]">
          qubit
        </text>
      </svg>
    </div>
  )
}
