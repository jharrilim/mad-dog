//! Multi-clock consistency runner for WASM.

use crate::models::{tfim_chain, tfim_cube, tfim_grid};
use crate::multi_clock::{build_multi_clock, MultiClockConfig, MultiClockResult};
use crate::quantum::QuantumState;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiClockRunConfig {
    #[serde(default = "default_kind")]
    pub kind: String,
    #[serde(default)]
    pub n: Option<usize>,
    #[serde(default)]
    pub rows: Option<usize>,
    #[serde(default)]
    pub cols: Option<usize>,
    #[serde(default)]
    pub lz: Option<usize>,
    pub field: f64,
    pub dt: f64,
    pub steps: usize,
    #[serde(default)]
    pub clock_sites: Option<Vec<usize>>,
    #[serde(default)]
    pub physical_slices: Option<usize>,
}

fn default_kind() -> String {
    "chain".to_string()
}

struct LatticeRun {
    kind: String,
    label: String,
    hamiltonian: crate::quantum::Hamiltonian,
    sites: usize,
    defect_site: usize,
    edge_sites: [usize; 2],
}

fn lattice_run(config: &MultiClockRunConfig) -> LatticeRun {
    match config.kind.as_str() {
        "grid" => {
            let rows = config.rows.unwrap_or(3);
            let cols = config.cols.unwrap_or(3);
            let model = tfim_grid(rows, cols, 1.0, config.field);
            let defect_site = (rows / 2) * cols + cols / 2;
            LatticeRun {
                kind: "grid".to_string(),
                label: format!("TFIM grid {rows}x{cols}"),
                hamiltonian: model.hamiltonian,
                sites: rows * cols,
                defect_site,
                edge_sites: [0, rows * cols - 1],
            }
        }
        "cube" => {
            let lx = config.rows.unwrap_or(2);
            let ly = config.cols.unwrap_or(2);
            let lz = config.lz.unwrap_or(3);
            let model = tfim_cube(lx, ly, lz, 1.0, config.field);
            let defect_site = (lz / 2) * (lx * ly) + (ly / 2) * lx + lx / 2;
            let sites = lx * ly * lz;
            LatticeRun {
                kind: "cube".to_string(),
                label: format!("TFIM cube {lx}x{ly}x{lz}"),
                hamiltonian: model.hamiltonian,
                sites,
                defect_site,
                edge_sites: [0, sites - 1],
            }
        }
        _ => {
            let n = config.n.unwrap_or(9);
            let model = tfim_chain(n, 1.0, config.field);
            LatticeRun {
                kind: "chain".to_string(),
                label: format!("TFIM chain n={n}"),
                hamiltonian: model.hamiltonian,
                sites: n,
                defect_site: n / 2,
                edge_sites: [0, n - 1],
            }
        }
    }
}

fn defect_initial(n: usize, center: usize) -> QuantumState {
    let mut initial = QuantumState::zero(n);
    initial.data[2 * (1 << center)] = 1.0;
    initial
}

fn reference_initial(n: usize) -> QuantumState {
    let mut reference = QuantumState::zero(n);
    reference.data[0] = 1.0;
    reference
}

pub fn run_multi_clock(config: &MultiClockRunConfig) -> MultiClockResult {
    let lattice = lattice_run(config);
    let initial = defect_initial(lattice.sites, lattice.defect_site);
    let reference = reference_initial(lattice.sites);
    build_multi_clock(
        &lattice.hamiltonian,
        initial,
        reference,
        &MultiClockConfig {
            n: lattice.sites,
            dt: config.dt,
            steps: config.steps,
            clock_sites: config.clock_sites.clone(),
            physical_slices: config.physical_slices.unwrap_or(15),
            defect_site: Some(lattice.defect_site),
            edge_sites: Some(lattice.edge_sites),
        },
        &lattice.kind,
        &lattice.label,
    )
}
