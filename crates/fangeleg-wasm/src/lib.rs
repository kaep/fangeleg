#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(target_arch = "wasm32")]
mod js_agent;

#[cfg(target_arch = "wasm32")]
pub use js_agent::{move_action, stay_action, tag_action};

#[cfg(target_arch = "wasm32")]
pub use wasm::*;
