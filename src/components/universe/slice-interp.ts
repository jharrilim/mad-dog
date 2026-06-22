import type { SpacetimeSlice } from '@/sim/types'

function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t
}

/** Interpolate coords, signal, and energy between adjacent clock slices. */
export function interpolateSlice(
  slices: SpacetimeSlice[],
  k: number,
): {
  coords: number[][]
  signal: number[]
  energy: number
  k0: number
  k1: number
  alpha: number
} {
  const k0 = Math.min(Math.floor(k), slices.length - 1)
  const k1 = Math.min(k0 + 1, slices.length - 1)
  const alpha = k0 === k1 ? 0 : k - k0
  const s0 = slices[k0]
  const s1 = slices[k1]
  const n = s0.coords.length

  const coords = Array.from({ length: n }, (_, i) => {
    const c0 = s0.coords[i]
    const c1 = s1.coords[i]
    const dims = Math.max(c0.length, c1.length, 1)
    return Array.from({ length: dims }, (_, d) =>
      lerp(c0[d] ?? 0, c1[d] ?? 0, alpha),
    )
  })

  const signal = s0.signal.map((v, i) => lerp(v, s1.signal[i] ?? v, alpha))
  const energy = lerp(s0.energy, s1.energy, alpha)

  return { coords, signal, energy, k0, k1, alpha }
}

export function lerpMiMatrix(
  a: number[][] | null,
  b: number[][] | null,
  t: number,
): number[][] | null {
  if (!a) return b
  if (!b || t <= 0) return a
  if (t >= 1) return b
  return a.map((row, i) =>
    row.map((v, j) => lerp(v, b[i]?.[j] ?? v, t)),
  )
}
