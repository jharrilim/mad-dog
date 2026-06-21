# Parameters and Phases

## Hamiltonian

```
H = -J Σ_<i,j> Z_i Z_j  -  h Σ_i X_i
```

| Symbol | Code | Meaning |
|--------|------|---------|
| `J` | `j` in `tfimChain/Grid/Cube` | Ising coupling — neighbours prefer aligned Z |
| `h` | `field` in UI / runners | Transverse field — mixes spin via X |

Default `J = 1`. Critical point for 1D TFIM: **h_c = 1** (for J=1).

## Phase diagram (intuition)

```
h → 0          h ≈ 1           h → ∞
ordered          critical        paramagnetic
(domains)                        (disordered in Z)
```

### Ordered side (h ≲ 1)

- Long-range Z correlations.
- Sharp **domain-wall** excitations.
- **Quench worldlines are clear** on the heatmap — localized propagating front.
- Ground-state MDS geometry can be **messier** (MI not locally decaying).

### Paramagnetic side (h ≳ 1)

- Transverse field dominates; Z correlations decay with distance.
- **Ground-state emergent geometry is clean** (why `sim-check` uses h=1.5).
- **Quench tracks muddy** — strong X mixing on all sites, signal smears, contrast drops.

## Practical guide

| Goal | Suggested h |
|------|-------------|
| Clean emergent **space** (ground state MI → MDS) | 1.0 – 1.5 |
| Sharp **worldlines** (quench / Universe Lab) | 0.6 – 1.0 |
| Near-critical physics | ~1.0 |

Universe Lab default h=1 is a compromise. User observation: **lower h → clearer worldline; higher h → muddier track** — consistent with ordered vs paramagnetic dynamics above.

## Other parameters

| Param | Typical | Role |
|-------|---------|------|
| `Δt` | 0.2–0.25 | Clock step; must keep `\|H\|·Δt` small enough for Taylor evolution |
| `steps` | 18–40 | Number of clock readings |
| `n` / lattice size | 8–12 | Hilbert dim = 2^n; exponential cost |

## Signal definition (why reference matters)

```
signal_i = |⟨Z_i⟩_defect − ⟨Z_i⟩_reference|
```

The transverse field causes ⟨Z⟩ to precess on **every** site, including the reference. Subtracting reference isolates the **perturbation** from uniform background dynamics.
