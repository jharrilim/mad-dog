# Emergent Particles (Conceptual)

## What a particle would be here

Not a primitive object — a **pattern** in the same fields we already plot:

1. **Localized** — energy/entropy disturbance confined to a small region of the emergent graph.
2. **Persistent** — recognizable lump across clock slices (identity over emergent time).
3. **Propagating** — moves along approximate geodesics / lattice paths at ≤ Lieb–Robinson speed.
4. **Classical (eventually)** — decoherence pins it to definite positions on branches.

## What we have (proto-particle)

The **central defect quench** in spacetime demos:

- Localized flip of one spin.
- Propagates as a disturbance with a **light cone** on the heatmap.
- In TFIM language: a **domain wall / spin excitation** — a quasiparticle, not a fundamental fermion.

**Worldline tracking (2026-06):** Rust tracks peak signal per slice (half-chain on 1D, global on cube); UI overlays on spacetime heatmaps, Universe Lab heatmap, and a 3D path through emergent MDS coordinates. See [scattering.md](./scattering.md).

**Two-defect scattering:** dual worldlines, separation-vs-time chart, velocity fits, falsification test E (propagate without binding). Experiments page → *Two-defect scattering*.

## What it looks like in each view

### Spacetime heatmap

A **worldline**: narrow bright band in site × time, plus solid centroid overlay. Clear in **ordered phase** (low h). Muddy in **paramagnetic phase** (high h) — see [parameters-and-phases.md](./parameters-and-phases.md).

### 3D Universe Lab

Scrubbing `k`: a **bright cluster** of nodes moves through emergent MDS positions; purple path = tracked worldline through emergent space.

### After decoherence (live demo)

- Before coupling: smeared mixed track (dashed worldline on heatmap).
- After env entanglement: **sharp branch-resolved tracks** per env outcome.
- Experiments → *Decoherence quench*; see [decoherence.md](./decoherence.md).
- **Subspace probe:** branch-resolved excitations show higher Pauli sharpness and lower window rank — see [qecc-probe.md](./qecc-probe.md).
- Essay page still has the static stepped diagram.

## What we do not have

- Stable labels (charge, spin, species).
- JW-resolved fermion exchange / phase shifts in scattering.
- QECC / infrared subspace identification — **proto-probe shipped** ([qecc-probe.md](./qecc-probe.md)); not a literal code ID yet.
- Automatic "particle detector" beyond signal centroid — would need richer observables + decoherence.

## Next experiments

See [roadmap.md](./roadmap.md). Near term:

1. Phase shift when two cones overlap (chain scattering, h sweep).
2. ~~Minimal **decoherence** → branch-resolved tracks.~~ **Done (2026-06)** — see [decoherence.md](./decoherence.md).
3. Measure **effective mass** from dispersion relation of lattice excitations.
