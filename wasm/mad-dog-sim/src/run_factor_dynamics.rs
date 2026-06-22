//! Phase 7 factor-count dynamics runners.

use crate::holographic_bound::{
    run_holographic_bound, HolographicBoundConfig, HolographicBoundRunResult,
};
use crate::tensor_split::{run_inplace_split, InplaceSplitConfig, InplaceSplitResult};

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InplaceSplitRunResult {
    #[serde(flatten)]
    pub inner: InplaceSplitResult,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

pub fn run_inplace_split_probe(config: &InplaceSplitConfig) -> InplaceSplitRunResult {
    InplaceSplitRunResult {
        inner: run_inplace_split(config),
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}

pub fn run_holographic_bound_probe(config: &HolographicBoundConfig) -> HolographicBoundRunResult {
    run_holographic_bound(config)
}
