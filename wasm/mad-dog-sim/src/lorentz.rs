//! Lorentz proxy metrics — directional front speeds on lattices.

use crate::run_spacetime::{run_spacetime_2d, Spacetime2DConfig};
use crate::spacetime::LightCone;

pub fn grid_manhattan_distances(rows: usize, cols: usize, center: usize) -> Vec<f64> {
    let r0 = center / cols;
    let c0 = center % cols;
    (0..rows * cols)
        .map(|site| {
            let r = site / cols;
            let c = site % cols;
            ((r as i32 - r0 as i32).unsigned_abs() + (c as i32 - c0 as i32).unsigned_abs()) as f64
        })
        .collect()
}

pub fn cardinal_neighbor_speeds(
    lc: &LightCone,
    rows: usize,
    cols: usize,
    center: usize,
) -> Vec<f64> {
    let r0 = center / cols;
    let c0 = center % cols;
    let neighbors = [
        (r0 as i32 - 1, c0 as i32),
        (r0 as i32 + 1, c0 as i32),
        (r0 as i32, c0 as i32 - 1),
        (r0 as i32, c0 as i32 + 1),
    ];
    let mut speeds = Vec::new();
    for (r, c) in neighbors {
        if r < 0 || r >= rows as i32 || c < 0 || c >= cols as i32 {
            continue;
        }
        let site = r as usize * cols + c as usize;
        let t = lc.arrivals[site];
        if t.is_finite() && t > 1e-9 {
            speeds.push(1.0 / t);
        }
    }
    speeds
}

pub fn speed_coefficient_of_variation(speeds: &[f64]) -> f64 {
    if speeds.len() < 2 {
        return f64::INFINITY;
    }
    let mean = speeds.iter().sum::<f64>() / speeds.len() as f64;
    if mean < 1e-9 {
        return f64::INFINITY;
    }
    let var = speeds.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / speeds.len() as f64;
    var.sqrt() / mean
}

pub fn grid_cardinal_speed_cv(rows: usize, cols: usize, field: f64, dt: f64, steps: usize) -> f64 {
    let result = run_spacetime_2d(&Spacetime2DConfig {
        rows,
        cols,
        field,
        dt,
        steps,
    });
    let lc = result
        .light_cone
        .expect("spacetime_2d attaches light_cone");
    let center = (rows / 2) * cols + cols / 2;
    let speeds = cardinal_neighbor_speeds(&lc, rows, cols, center);
    speed_coefficient_of_variation(&speeds)
}

pub fn grid_light_cone_velocity(
    rows: usize,
    cols: usize,
    field: f64,
    dt: f64,
    steps: usize,
    signal_reference_site: Option<usize>,
) -> f64 {
    use crate::models::tfim_grid;
    use crate::quantum::QuantumState;
    use crate::spacetime::{build_spacetime, measure_light_cone, SpacetimeConfig};

    let model = tfim_grid(rows, cols, 1.0, field);
    let n = rows * cols;
    let center = (rows / 2) * cols + cols / 2;
    let mut initial = QuantumState::zero(n);
    initial.data[2 * (1 << center)] = 1.0;
    let mut reference = QuantumState::zero(n);
    reference.data[0] = 1.0;
    let result = build_spacetime(SpacetimeConfig {
        hamiltonian: &model.hamiltonian,
        initial,
        reference: Some(reference),
        dt,
        steps,
        embed_dim: 2,
        align_to: Some(&model.layout.true_positions),
        order: 6,
        include_geometry: false,
        track_energy: false,
        retain_slices: true,
        worldline_defects: None,
        track_worldline: false,
        defect_site: Some(center),
        seed: 42,
        signal_reference_site,
    });
    let distances = grid_manhattan_distances(rows, cols, center);
    measure_light_cone(&result, 0.12, Some(center), Some(&distances)).velocity
}
