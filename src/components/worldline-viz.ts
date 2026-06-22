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
  maxSlice?: number,
): [number, number, number][] {
  const end = Math.min(maxSlice ?? worldline.length - 1, worldline.length - 1)
  return worldline.slice(0, end + 1).map((p, k) => {
    const c = slices[k]?.coords[p.site] ?? [0, 0, 0]
    return [c[0], c[1] ?? 0, c[2] ?? 0]
  })
}

/** Worldline path with a fractional tip between slices k₀ and k₀+1. */
export function worldlineEmergentPathTweened(
  worldline: WorldlinePoint[],
  slices: { coords: number[][] }[],
  k: number,
): [number, number, number][] {
  const k0 = Math.min(Math.floor(k), worldline.length - 1)
  const path = worldlineEmergentPath(worldline, slices, k0)
  if (k0 >= worldline.length - 1 || k <= k0) return path

  const alpha = k - k0
  const next = worldline[k0 + 1]
  const c = slices[k0 + 1]?.coords[next.site] ?? [0, 0, 0]
  const tip: [number, number, number] = [c[0], c[1] ?? 0, c[2] ?? 0]
  if (path.length === 0) return [tip]

  const prev = path[path.length - 1]
  return [
    ...path,
    [
      prev[0] + (tip[0] - prev[0]) * alpha,
      prev[1] + (tip[1] - prev[1]) * alpha,
      prev[2] + (tip[2] - prev[2]) * alpha,
    ],
  ]
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
