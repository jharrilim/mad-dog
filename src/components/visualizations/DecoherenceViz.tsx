import { useState } from 'react'
import { decoherenceSteps } from '@/data'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { ChevronLeft, ChevronRight } from 'lucide-react'

export function DecoherenceViz() {
  const [step, setStep] = useState(0)
  const current = decoherenceSteps[step]

  return (
    <Card>
      <CardHeader>
        <CardTitle>Decoherence &amp; Branching</CardTitle>
        <CardDescription>
          How measurement and environmental entanglement produce classical-looking
          branches in Everettian quantum mechanics
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex items-center justify-between">
          <Button
            variant="outline"
            size="sm"
            onClick={() => setStep((s) => Math.max(0, s - 1))}
            disabled={step === 0}
          >
            <ChevronLeft className="size-4" />
            Previous
          </Button>
          <span className="text-sm text-muted-foreground">
            Step {step + 1} of {decoherenceSteps.length}:{' '}
            <span className="text-foreground font-medium">{current.label}</span>
          </span>
          <Button
            variant="outline"
            size="sm"
            onClick={() =>
              setStep((s) => Math.min(decoherenceSteps.length - 1, s + 1))
            }
            disabled={step === decoherenceSteps.length - 1}
          >
            Next
            <ChevronRight className="size-4" />
          </Button>
        </div>

        <div className="rounded-lg border bg-muted/30 p-4 font-mono text-sm leading-relaxed overflow-x-auto">
          {current.equation}
        </div>

        <p className="text-sm text-muted-foreground leading-relaxed">
          {current.description}
        </p>

        <svg
          viewBox="0 0 480 120"
          className="w-full h-auto"
          role="img"
          aria-label="Decoherence branching diagram"
        >
          {/* Subsystem boxes */}
          {[
            { label: 'Object (q)', x: 30, color: 'oklch(0.72 0.16 290)' },
            { label: 'Apparatus (a)', x: 170, color: 'oklch(0.65 0.14 200)' },
            { label: 'Environment (e)', x: 310, color: 'oklch(0.6 0.12 160)' },
          ].map((sys) => {
            const entangled =
              (sys.label.startsWith('Object') && step >= 0) ||
              (sys.label.startsWith('Apparatus') && step >= 1) ||
              (sys.label.startsWith('Environment') && step >= 2)
            const opacity = entangled ? 1 : 0.35
            return (
              <g key={sys.label} opacity={opacity}>
                <rect
                  x={sys.x}
                  y="30"
                  width="120"
                  height="50"
                  rx="8"
                  fill={sys.color}
                  fillOpacity={0.15}
                  stroke={sys.color}
                  strokeWidth="2"
                />
                <text
                  x={sys.x + 60}
                  y="60"
                  textAnchor="middle"
                  className="fill-foreground text-[11px]"
                >
                  {sys.label}
                </text>
              </g>
            )
          })}

          {/* Entanglement links */}
          {step >= 1 && (
            <line
              x1="150"
              y1="55"
              x2="170"
              y2="55"
              stroke="oklch(0.72 0.16 290)"
              strokeWidth="2"
              strokeDasharray="4 3"
            />
          )}
          {step >= 2 && (
            <line
              x1="290"
              y1="55"
              x2="310"
              y2="55"
              stroke="oklch(0.65 0.14 200)"
              strokeWidth="2"
              strokeDasharray="4 3"
            />
          )}

          {/* Branch split */}
          {step === 2 && current.branches && (
            <>
              <path
                d="M 60 95 Q 120 85 200 95"
                fill="none"
                stroke="oklch(0.72 0.16 290)"
                strokeWidth="2"
              />
              <path
                d="M 60 95 Q 120 105 200 105"
                fill="none"
                stroke="oklch(0.55 0.14 200)"
                strokeWidth="2"
              />
              {current.branches.map((branch, i) => (
                <text
                  key={branch.label}
                  x={220 + i * 120}
                  y={i === 0 ? 98 : 108}
                  className="fill-muted-foreground text-[10px] font-mono"
                >
                  {branch.label}: p = {branch.amplitude}
                </text>
              ))}
            </>
          )}
        </svg>
      </CardContent>
    </Card>
  )
}
