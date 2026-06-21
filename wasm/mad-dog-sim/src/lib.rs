mod emergence;
mod factorization;
mod geometry;
mod holography;
mod linalg;
mod models;
mod quantum;
mod relational_time;
mod rng;
mod refinement;
mod run_factorization;
mod run_factorization_refinement;
mod run_falsification;
mod run_holography;
mod modular_time;
mod run_light_cone_compare;
mod run_modular_dual_clock;
mod run_refinement;
mod scattering;
mod run_relational_time;
mod run_spacetime;
mod run_universe;
mod spacetime;

use emergence::{run_emergence, RunConfig as EmergenceConfig};
use run_factorization::{run_factorization_search, FactorizationSearchConfig};
use run_factorization_refinement::{
    run_factorization_refinement_study, FactorizationRefinementConfig,
};
use run_falsification::run_falsification_battery;
use run_holography::{
    run_holography, run_rt_mass, HolographyRunConfig, RtMassRunConfig,
};
use run_light_cone_compare::{run_light_cone_compare, LightConeCompareConfig};
use run_modular_dual_clock::run_modular_dual_clock;
use modular_time::ModularDualClockConfig;
use scattering::{run_two_defect_scattering, ScatteringConfig};
use run_refinement::{
    run_refinement_n_compare, run_refinement_quench, RefinementNCompareConfig,
    RefinementQuenchConfig,
};
use run_relational_time::{run_relational_time, RelationalTimeConfig};
use run_spacetime::{run_spacetime, run_spacetime_2d, Spacetime2DConfig, SpacetimeRunConfig};
use run_universe::{
    run_universe_3d, run_universe_slice, Universe3DConfig, UniverseSliceConfig,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run_emergence_json(config_json: &str) -> Result<String, JsValue> {
    let start = js_sys::Date::now();
    let config: EmergenceConfig =
        serde_json::from_str(config_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let mut result = run_emergence(&config);
    result.elapsed_ms = js_sys::Date::now() - start;
    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn run_spacetime_json(config_json: &str) -> Result<String, JsValue> {
    let start = js_sys::Date::now();
    let config: SpacetimeRunConfig =
        serde_json::from_str(config_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let mut result = run_spacetime(&config);
    result.elapsed_ms = js_sys::Date::now() - start;
    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn run_spacetime_2d_json(config_json: &str) -> Result<String, JsValue> {
    let start = js_sys::Date::now();
    let config: Spacetime2DConfig =
        serde_json::from_str(config_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let mut result = run_spacetime_2d(&config);
    result.elapsed_ms = js_sys::Date::now() - start;
    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn run_holography_json(config_json: &str) -> Result<String, JsValue> {
    let start = js_sys::Date::now();
    let config: HolographyRunConfig =
        serde_json::from_str(config_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let mut result = run_holography(&config);
    result.elapsed_ms = js_sys::Date::now() - start;
    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn run_rt_mass_json(config_json: &str) -> Result<String, JsValue> {
    let start = js_sys::Date::now();
    let config: RtMassRunConfig =
        serde_json::from_str(config_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let mut result = run_rt_mass(&config);
    result.elapsed_ms = js_sys::Date::now() - start;
    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn run_light_cone_compare_json(config_json: &str) -> Result<String, JsValue> {
    let start = js_sys::Date::now();
    let config: LightConeCompareConfig =
        serde_json::from_str(config_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let mut result = run_light_cone_compare(&config);
    result.elapsed_ms = js_sys::Date::now() - start;
    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn run_modular_dual_clock_json(config_json: &str) -> Result<String, JsValue> {
    let start = js_sys::Date::now();
    let config: ModularDualClockConfig =
        serde_json::from_str(config_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let mut result = run_modular_dual_clock(&config);
    result.elapsed_ms = js_sys::Date::now() - start;
    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn run_scattering_json(config_json: &str) -> Result<String, JsValue> {
    let start = js_sys::Date::now();
    let config: ScatteringConfig =
        serde_json::from_str(config_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let mut result = run_two_defect_scattering(&config);
    result.elapsed_ms = js_sys::Date::now() - start;
    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn run_refinement_quench_json(config_json: &str) -> Result<String, JsValue> {
    let start = js_sys::Date::now();
    let config: RefinementQuenchConfig =
        serde_json::from_str(config_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let mut result = run_refinement_quench(&config);
    result.elapsed_ms = js_sys::Date::now() - start;
    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn run_refinement_n_compare_json(config_json: &str) -> Result<String, JsValue> {
    let start = js_sys::Date::now();
    let config: RefinementNCompareConfig =
        serde_json::from_str(config_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let mut result = run_refinement_n_compare(&config);
    result.elapsed_ms = js_sys::Date::now() - start;
    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn run_relational_time_json(config_json: &str) -> Result<String, JsValue> {
    let start = js_sys::Date::now();
    let config: RelationalTimeConfig =
        serde_json::from_str(config_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let mut result = run_relational_time(&config);
    result.elapsed_ms = js_sys::Date::now() - start;
    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn run_universe_3d_json(config_json: &str) -> Result<String, JsValue> {
    let start = js_sys::Date::now();
    let config: Universe3DConfig =
        serde_json::from_str(config_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let mut result = run_universe_3d(&config);
    result.elapsed_ms = js_sys::Date::now() - start;
    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn run_universe_slice_json(config_json: &str) -> Result<String, JsValue> {
    let start = js_sys::Date::now();
    let config: UniverseSliceConfig =
        serde_json::from_str(config_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let mut result = run_universe_slice(&config);
    result.elapsed_ms = js_sys::Date::now() - start;
    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn run_factorization_search_json(config_json: &str) -> Result<String, JsValue> {
    let start = js_sys::Date::now();
    let config: FactorizationSearchConfig =
        serde_json::from_str(config_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let mut result = run_factorization_search(&config);
    result.elapsed_ms = js_sys::Date::now() - start;
    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn run_factorization_refinement_json(config_json: &str) -> Result<String, JsValue> {
    let start = js_sys::Date::now();
    let config: FactorizationRefinementConfig =
        serde_json::from_str(config_json).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let mut result = run_factorization_refinement_study(&config);
    result.elapsed_ms = js_sys::Date::now() - start;
    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn run_falsification_battery_json(_config_json: &str) -> Result<String, JsValue> {
    let start = js_sys::Date::now();
    let mut result = run_falsification_battery();
    result.elapsed_ms = js_sys::Date::now() - start;
    serde_json::to_string(&result).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn wasm_sim_version() -> String {
    "0.7.0".to_string()
}

#[cfg(test)]
mod tests {
    use crate::rng::Rng;

    #[test]
    fn rng_sequence_seed_1768() {
        let a = 1768u32.wrapping_add(0x6d2b79f5) as i32;
        assert_eq!(a, 1831567581);
        let mut rng = Rng::new(1768);
        let v = rng.next();
        assert!((v - 0.122_922_855_895_012_6).abs() < 1e-10, "got {v}");
    }

    #[test]
    fn rng_matches_mulberry32_first_values() {
        let mut rng = Rng::new(12345);
        let a = rng.next();
        assert!((a - 0.979_728_267_760_947_3).abs() < 1e-10);
    }
}
