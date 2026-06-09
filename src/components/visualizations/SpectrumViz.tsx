import { energyLevels } from '@/data'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'

function formatComplex(re: number, im: number): string {
  const sign = im >= 0 ? '+' : '−'
  return `${re.toFixed(2)} ${sign} ${Math.abs(im).toFixed(2)}i`
}

export function SpectrumViz() {
  const maxEnergy = Math.max(...energyLevels.map((l) => l.energy))
  const maxProb = Math.max(
    ...energyLevels.map((l) => l.amplitude.re ** 2 + l.amplitude.im ** 2),
  )

  return (
    <Card>
      <CardHeader>
        <CardTitle>Minimal Data</CardTitle>
        <CardDescription>
          The Hamiltonian spectrum {'{Eₙ}'} and state components {'{ψₙ}'} in
          the energy eigenbasis
        </CardDescription>
      </CardHeader>
      <CardContent>
        <svg
          viewBox="0 0 480 260"
          className="w-full h-auto"
          role="img"
          aria-label="Energy levels and state amplitudes"
        >
          <defs>
            <linearGradient id="energyGrad" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stopColor="oklch(0.72 0.16 290)" />
              <stop offset="100%" stopColor="oklch(0.55 0.14 200)" />
            </linearGradient>
          </defs>

          {/* Energy levels */}
          <text x="10" y="20" className="fill-muted-foreground text-[11px]">
            Energy Eₙ
          </text>
          {energyLevels.map((level) => {
            const barWidth = (level.energy / maxEnergy) * 180
            const y = 40 + level.n * 32
            return (
              <g key={`energy-${level.n}`}>
                <text
                  x="10"
                  y={y + 4}
                  className="fill-foreground text-[11px] font-mono"
                >
                  E{level.n}
                </text>
                <rect
                  x="50"
                  y={y - 8}
                  width={barWidth}
                  height="16"
                  rx="3"
                  fill="url(#energyGrad)"
                  opacity={0.85}
                />
                <text
                  x={55 + barWidth}
                  y={y + 4}
                  className="fill-muted-foreground text-[10px] font-mono"
                >
                  {level.energy.toFixed(1)}
                </text>
              </g>
            )
          })}

          {/* Amplitude magnitudes */}
          <text x="260" y="20" className="fill-muted-foreground text-[11px]">
            |ψₙ|² (Born weight)
          </text>
          {energyLevels.map((level) => {
            const prob =
              level.amplitude.re ** 2 + level.amplitude.im ** 2
            const barWidth = (prob / maxProb) * 140
            const y = 40 + level.n * 32
            return (
              <g key={`prob-${level.n}`}>
                <text
                  x="260"
                  y={y + 4}
                  className="fill-foreground text-[11px] font-mono"
                >
                  ψ{level.n}
                </text>
                <rect
                  x="300"
                  y={y - 8}
                  width={barWidth}
                  height="16"
                  rx="3"
                  className="fill-accent"
                  opacity={0.7}
                />
                <text
                  x={305 + barWidth}
                  y={y + 4}
                  className="fill-muted-foreground text-[10px] font-mono"
                >
                  {prob.toFixed(3)}
                </text>
              </g>
            )
          })}

          {/* Complex components legend */}
          <text x="10" y="248" className="fill-muted-foreground text-[10px]">
            ψₙ components:{' '}
            {energyLevels
              .slice(0, 3)
              .map((l) => `ψ${l.n}=${formatComplex(l.amplitude.re, l.amplitude.im)}`)
              .join(', ')}
            …
          </text>
        </svg>
      </CardContent>
    </Card>
  )
}
