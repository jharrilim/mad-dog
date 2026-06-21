mod emergence;
mod geometry;
mod holography;
mod linalg;
mod models;
mod quantum;
mod rng;
mod run_holography;
mod run_spacetime;
mod spacetime;

use emergence::{run_emergence, RunConfig as EmergenceConfig};
use run_holography::{
    run_holography, run_rt_mass, HolographyRunConfig, RtMassRunConfig,
};
use run_spacetime::{run_spacetime, run_spacetime_2d, Spacetime2DConfig, SpacetimeRunConfig};
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
pub fn wasm_sim_version() -> String {
    "0.2.0".to_string()
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
