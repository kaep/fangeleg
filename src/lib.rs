pub mod competent_agent;
pub mod naive_agent;
pub mod simulation;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(target_arch = "wasm32")]
pub mod js_agent;
