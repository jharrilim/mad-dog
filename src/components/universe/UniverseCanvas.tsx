import { useEffect, useMemo, useRef } from 'react'
import { Canvas } from '@react-three/fiber'
import { OrbitControls, Line } from '@react-three/drei'
import * as THREE from 'three'
import type { Universe3DResult } from '@/sim/runner'
import type { SpacetimeSlice } from '@/sim/types'
import { worldlineEmergentPath } from '@/components/worldline-viz'

interface Bounds {
  cx: number
  cy: number
  cz: number
  span: number
}

function computeBounds(slices: SpacetimeSlice[]): Bounds {
  let minX = Infinity
  let maxX = -Infinity
  let minY = Infinity
  let maxY = -Infinity
  let minZ = Infinity
  let maxZ = -Infinity
  for (const slice of slices) {
    for (const c of slice.coords) {
      minX = Math.min(minX, c[0])
      maxX = Math.max(maxX, c[0])
      minY = Math.min(minY, c[1] ?? 0)
      maxY = Math.max(maxY, c[1] ?? 0)
      minZ = Math.min(minZ, c[2] ?? 0)
      maxZ = Math.max(maxZ, c[2] ?? 0)
    }
  }
  const cx = (minX + maxX) / 2
  const cy = (minY + maxY) / 2
  const cz = (minZ + maxZ) / 2
  const span =
    Math.max(maxX - minX, maxY - minY, maxZ - minZ, 1e-6) || 1
  return { cx, cy, cz, span }
}

function toScene(
  x: number,
  y: number,
  z: number,
  b: Bounds,
  scale = 3.5,
): [number, number, number] {
  const s = scale / b.span
  return [(x - b.cx) * s, (y - b.cy) * s, (z - b.cz) * s]
}

function signalColor(intensity: number): THREE.Color {
  const c = new THREE.Color()
  c.setHSL(0.72 - intensity * 0.08, 0.65, 0.45 + intensity * 0.25)
  return c
}

function UniverseScene({
  result,
  selected,
  mi,
}: {
  result: Universe3DResult
  selected: number
  mi: number[][] | null
}) {
  const slice = result.spacetime.slices[selected]
  const bounds = useMemo(
    () => computeBounds(result.spacetime.slices),
    [result.spacetime.slices],
  )

  let maxSig = 1e-9
  for (const s of result.spacetime.slices) {
    for (const v of s.signal) maxSig = Math.max(maxSig, v)
  }

  let maxMi = 1e-9
  if (mi) {
    for (let i = 0; i < mi.length; i++) {
      for (let j = i + 1; j < mi.length; j++) {
        maxMi = Math.max(maxMi, mi[i][j])
      }
    }
  }

  const coneThr = maxSig * 0.15

  const trackedSite =
    result.spacetime.worldline?.[selected]?.site ?? result.defectSite

  const worldlinePath = useMemo(() => {
    const wl = result.spacetime.worldline
    if (!wl || wl.length === 0) return null
    const raw = worldlineEmergentPath(wl, result.spacetime.slices, selected)
    return raw.map(([x, y, z]) => toScene(x, y, z, bounds))
  }, [result.spacetime.worldline, result.spacetime.slices, bounds, selected])

  return (
    <>
      <ambientLight intensity={0.45} />
      <directionalLight position={[4, 6, 3]} intensity={1.1} />
      <directionalLight position={[-3, -2, -4]} intensity={0.35} />

      {result.edges.map(([i, j], e) => {
        const a = slice.coords[i]
        const b = slice.coords[j]
        const pa = toScene(a[0], a[1] ?? 0, a[2] ?? 0, bounds)
        const pb = toScene(b[0], b[1] ?? 0, b[2] ?? 0, bounds)
        const miVal = mi ? mi[i][j] / maxMi : 0.35
        return (
          <Line
            key={e}
            points={[pa, pb]}
            color="#6b5b8a"
            transparent
            opacity={0.15 + miVal * 0.75}
            lineWidth={1}
          />
        )
      })}

      {worldlinePath && worldlinePath.length > 1 && (
        <Line
          points={worldlinePath}
          color="#9b6dff"
          lineWidth={2.5}
          transparent
          opacity={0.9}
        />
      )}

      {slice.coords.map((c, i) => {
        const intensity = slice.signal[i] / maxSig
        const pos = toScene(c[0], c[1] ?? 0, c[2] ?? 0, bounds)
        const isTrackHead = i === trackedSite
        const inCone = intensity > coneThr
        const radius =
          (isTrackHead ? 0.16 : 0.09) + intensity * (isTrackHead ? 0.12 : 0.1)
        return (
          <group key={i} position={pos}>
            {inCone && (
              <mesh>
                <sphereGeometry args={[radius * 1.8, 16, 16]} />
                <meshBasicMaterial
                  color={signalColor(intensity)}
                  transparent
                  opacity={0.12}
                />
              </mesh>
            )}
            <mesh>
              <sphereGeometry args={[radius, 20, 20]} />
              <meshStandardMaterial
                color={signalColor(intensity)}
                emissive={signalColor(intensity)}
                emissiveIntensity={0.15 + intensity * 0.5}
                metalness={0.2}
                roughness={0.45}
              />
            </mesh>
          </group>
        )
      })}

      <OrbitControls
        enableDamping
        dampingFactor={0.08}
        enablePan={false}
      />
    </>
  )
}

export function UniverseCanvas({
  result,
  selected,
  mi,
}: {
  result: Universe3DResult
  selected: number
  mi: number[][] | null
}) {
  const containerRef = useRef<HTMLDivElement>(null)

  // Keep OrbitControls zoom from chaining into page scroll.
  useEffect(() => {
    const el = containerRef.current
    if (!el) return
    const blockScroll = (e: WheelEvent) => e.preventDefault()
    el.addEventListener('wheel', blockScroll, { passive: false })
    return () => el.removeEventListener('wheel', blockScroll)
  }, [])

  return (
    <div
      ref={containerRef}
      className="w-full h-[min(420px,55vh)] shrink-0 rounded-lg overflow-hidden border bg-black/40"
      style={{ touchAction: 'none' }}
    >
      <Canvas
        camera={{ position: [5, 4, 5], fov: 45 }}
        gl={{ antialias: true }}
        style={{ width: '100%', height: '100%', display: 'block' }}
      >
        <UniverseScene result={result} selected={selected} mi={mi} />
      </Canvas>
    </div>
  )
}
