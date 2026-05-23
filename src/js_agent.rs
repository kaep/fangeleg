use crate::simulation::{Agent, AgentAction, AgentInput};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct JsAgent {
    act_function: js_sys::Function,
}

#[wasm_bindgen]
impl JsAgent {
    pub fn new(act_function: js_sys::Function) -> Self {
        Self { act_function }
    }
}

impl Agent for JsAgent {
    fn act(&self, _input: AgentInput) -> AgentAction {
        web_sys::console::log_1(&JsValue::from_str("Hi from wasm agent, rust side"));

        if let Err(error) = self.act_function.call0(&JsValue::NULL) {
            web_sys::console::error_1(&error);
        }
        AgentAction::Stay
    }
}
