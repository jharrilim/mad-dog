import type { WorldlinePoint } from '@/sim/types'

export const WORLDLINE_COLORS = [
  'oklch(0.72 0.16 290)',
  'oklch(0.72 0.14 155)',
] as const

export interface HeatmapLayout {
  cell: number
  padL: number
  padT: number
}

/** Map worldline points to SVG polyline coordinates on a site×time heatmap. */
export function worldlinePolylinePoints(
  worldline: WorldlinePoint[],
  layout: HeatmapLayout,
  sliceIndex?: (k: number, point: WorldlinePoint) => number,
): string {
  const { cell, padL, padT } = layout
  return worldline
    .map((p, k) => {
      const row = sliceIndex ? sliceIndex(k, p) : k
      const x = padL + p.site * cell + cell / 2
      const y = padT + row * cell + cell / 2
      return `${x},${y}`
    })
    .join(' ')
}

/** Emergent 3D positions along a tracked worldline (one point per slice). */
export function worldlineEmergentPath(
  worldline: WorldlinePoint[],
  slices: { coords: number[][] }[],
): [number, number, number][] {
  return worldline.map((p, k) => {
    const c = slices[k]?.coords[p.site] ?? [0, 0, 0]
    return [c[0], c[1] ?? 0, c[2] ?? 0]
  })
}

/** Mini SVG separation-vs-time chart for two-defect scattering. */
export function separationChartPath(
  series: number[],
  width: number,
  height: number,
  pad = 8,
): { line: string; minY: number; maxY: number } {
  if (series.length === 0) {
    return { line: '', minY: 0, maxY: 1 }
  }
  const minY = 0
  const maxY = Math.max(...series, 1)
  const innerW = width - 2 * pad
  const innerH = height - 2 * pad
  const line = series
    .map((sep, k) => {
      const x = pad + (k / Math.max(series.length - 1, 1)) * innerW
      const y = pad + innerH - (sep / maxY) * innerH
      return `${x},${y}`
    })
    .join(' ')
  return { line, minY, maxY }
}
