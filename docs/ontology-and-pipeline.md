# Ontology and Pipeline

## Mad-Dog starting point

The paper's austere ontology:

1. A Hilbert space ℋ (here: finite, factored as ⊗ₐ ℋₐ).
2. A state vector |ψ⟩.
3. A Hamiltonian Ĥ (spectrum + state components in the energy basis = primitive data).

No built-in space, time, particles, or classical variables. Those should **emerge** from entanglement structure and dynamics on a suitable factorization.

## What we actually simulate

| Layer | Implementation |
|-------|----------------|
| Hilbert space | `2^n` complex amplitudes (`QuantumState`) |
| Hamiltonian | Pauli-sum `H = Σ coeff · (tensor product of X,Y,Z on sites)` |
| Low-energy states | Imaginary-time evolution `(1 - dt·H)` + normalize |
| Real-time dynamics | Taylor series `e^{-iHdt}` with adaptive substepping |

## Emergence pipeline (Cao–Carroll–Michalakis style)

Referenced in code comments (arXiv:1606.08444):

```
|ψ⟩  →  mutual information I(a:b)  →  distance d(a,b) = -ξ ln(I/I_max)
     →  classical MDS  →  coordinates + eigenvalue spectrum (emergent dim)
```

For dynamics:

```
|ψ₀⟩ quench  →  Schrödinger evolution  →  slice k at t = k·Δt
             →  same geometry pipeline per slice  →  emergent spacetime
```

## Design choice: correctness first

The reference engine is plain TypeScript over `Float64Array`. It is intentionally small and auditable. Size limits:

| System | Qubits | Hilbert dim | Status |
|--------|--------|-------------|--------|
| Chain | 8–12 | 256–4096 | Comfortable |
| Grid 3×3 | 9 | 512 | Comfortable |
| Cube 2×2×2 | 8 | 256 | Comfortable |
| Cube 2×2×3 | 12 | 4096 | Slower |
| Cube 3×3×3 | 27 | ~134M | Out of scope (for now) |

WebGPU/WASM deferred; `runner.ts` is the boundary so the backend can be swapped later.
