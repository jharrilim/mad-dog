import { useEffect, useRef, useState } from 'react'

function easeOutCubic(t: number): number {
  return 1 - (1 - t) ** 3
}

/**
 * Smoothly animate a fractional slice index toward an integer target.
 * Pass `resetKey` (e.g. simulation result) to snap without tweening on new runs.
 */
export function useSliceTween(
  target: number,
  durationMs = 300,
  resetKey?: unknown,
): number {
  const [display, setDisplay] = useState(target)
  const displayRef = useRef(target)
  const rafRef = useRef<number | null>(null)

  useEffect(() => {
    displayRef.current = target
    setDisplay(target)
    // Snap to target when simulation resets; target is from the same render as resetKey.
    // eslint-disable-next-line react-hooks/exhaustive-deps -- only snap on new runs
  }, [resetKey])

  useEffect(() => {
    const to = target
    const from = displayRef.current
    if (Math.abs(to - from) < 1e-6) return

    const start = performance.now()

    const tick = (now: number) => {
      const t = Math.min(1, (now - start) / durationMs)
      const v = from + (to - from) * easeOutCubic(t)
      displayRef.current = v
      setDisplay(v)
      if (t < 1) {
        rafRef.current = requestAnimationFrame(tick)
      }
    }

    rafRef.current = requestAnimationFrame(tick)
    return () => {
      if (rafRef.current !== null) cancelAnimationFrame(rafRef.current)
    }
  }, [target, durationMs])

  return display
}
