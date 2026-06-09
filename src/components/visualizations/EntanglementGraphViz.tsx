import { graphEdges, graphNodes } from '@/data'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'

function getNode(id: string) {
  const node = graphNodes.find((n) => n.id === id)
  if (!node) throw new Error(`Node ${id} not found`)
  return node
}

export function EntanglementGraphViz() {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Emergent Geometry</CardTitle>
        <CardDescription>
          Hilbert-space micro-factors connected by mutual information. Stronger
          entanglement → shorter emergent distance.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <svg
          viewBox="0 0 520 360"
          className="w-full h-auto"
          role="img"
          aria-label="Entanglement graph with mutual information weights"
        >
          <defs>
            <filter id="glow">
              <feGaussianBlur stdDeviation="2" result="blur" />
              <feMerge>
                <feMergeNode in="blur" />
                <feMergeNode in="SourceGraphic" />
              </feMerge>
            </filter>
          </defs>

          {/* Edges */}
          {graphEdges.map((edge) => {
            const from = getNode(edge.from)
            const to = getNode(edge.to)
            const strokeWidth = 1 + edge.mutualInfo * 5
            const opacity = 0.3 + edge.mutualInfo * 0.6
            const midX = (from.x + to.x) / 2
            const midY = (from.y + to.y) / 2
            return (
              <g key={`${edge.from}-${edge.to}`}>
                <line
                  x1={from.x}
                  y1={from.y}
                  x2={to.x}
                  y2={to.y}
                  stroke="oklch(0.55 0.14 200)"
                  strokeWidth={strokeWidth}
                  opacity={opacity}
                  strokeLinecap="round"
                />
                <text
                  x={midX}
                  y={midY - 6}
                  textAnchor="middle"
                  className="fill-muted-foreground text-[9px] font-mono"
                >
                  I={edge.mutualInfo.toFixed(2)}
                </text>
              </g>
            )
          })}

          {/* Nodes */}
          {graphNodes.map((node) => (
            <g key={node.id} filter="url(#glow)">
              <circle
                cx={node.x}
                cy={node.y}
                r="22"
                className="fill-card stroke-primary"
                strokeWidth="2"
              />
              <text
                x={node.x}
                y={node.y + 5}
                textAnchor="middle"
                className="fill-foreground text-[14px] font-serif"
              >
                {node.label}
              </text>
              <text
                x={node.x}
                y={node.y + 36}
                textAnchor="middle"
                className="fill-muted-foreground text-[9px]"
              >
                H_{node.label}
              </text>
            </g>
          ))}

          {/* Legend */}
          <rect x="20" y="310" width="200" height="40" rx="6" className="fill-muted/50" />
          <line x1="35" y1="325" x2="75" y2="325" stroke="oklch(0.55 0.14 200)" strokeWidth="5" opacity="0.9" />
          <text x="85" y="329" className="fill-muted-foreground text-[10px]">
            High I → short distance
          </text>
          <line x1="35" y1="345" x2="75" y2="345" stroke="oklch(0.55 0.14 200)" strokeWidth="1.5" opacity="0.4" />
          <text x="85" y="349" className="fill-muted-foreground text-[10px]">
            Low I → long distance
          </text>
        </svg>
      </CardContent>
    </Card>
  )
}
