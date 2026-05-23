use crate::simulation::{Agent, AgentAction, AgentInput, Point2D};
use wasm_bindgen::prelude::*;

const ACTION_STAY: u8 = 0;
const ACTION_MOVE: u8 = 1;
const ACTION_TAG: u8 = 2;

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
    fn act(&self, input: AgentInput) -> AgentAction {
        let wasm_input = WasmAgentInput::from(input);
        let wasm_input_js = JsValue::from(wasm_input);

        let Ok(js_action) = self.act_function.call1(&JsValue::NULL, &wasm_input_js) else {
            web_sys::console::error_1(&JsValue::from_str(
                "JS agent act function call failed, deciding to stay",
            ));
            return AgentAction::Stay;
        };

        js_value_to_agent_action(js_action).unwrap_or_else(|| {
            web_sys::console::error_1(&JsValue::from_str(
                "JS agent act function returned an invalid action deciding to stay",
            ));
            AgentAction::Stay
        })
    }
}

#[wasm_bindgen]
pub struct WasmPoint {
    point: Point2D,
}

#[wasm_bindgen]
impl WasmPoint {
    #[wasm_bindgen(getter)]
    pub fn x(&self) -> usize {
        self.point.x
    }

    #[wasm_bindgen(getter)]
    pub fn y(&self) -> usize {
        self.point.y
    }
}

impl From<Point2D> for WasmPoint {
    fn from(point: Point2D) -> Self {
        Self { point }
    }
}

impl From<WasmPoint> for Point2D {
    fn from(wasm_point: WasmPoint) -> Self {
        wasm_point.point
    }
}

#[derive(Debug)]
#[wasm_bindgen]
pub struct WasmAgentInput {
    input: AgentInput,
}

#[wasm_bindgen]
impl WasmAgentInput {
    #[wasm_bindgen(getter)]
    pub fn position(&self) -> WasmPoint {
        self.input.position.into()
    }

    #[wasm_bindgen(js_name = taggablePositionCount)]
    pub fn taggable_position_count(&self) -> usize {
        self.input.taggable_positions.len()
    }

    #[wasm_bindgen(js_name = taggablePositionAt)]
    pub fn taggable_position_at(&self, index: usize) -> Option<WasmPoint> {
        self.input
            .taggable_positions
            .get(index)
            .copied()
            .map(WasmPoint::from)
    }
}

impl From<AgentInput> for WasmAgentInput {
    fn from(input: AgentInput) -> Self {
        Self { input }
    }
}

// Creating a wasm wrapper for AgentAction is possible but will not be wasm_bindgen::JsCast
// so conversion is done manually
fn js_value_to_agent_action(value: JsValue) -> Option<AgentAction> {
    if !js_sys::Array::is_array(&value) {
        return None;
    }

    let array = js_sys::Array::from(&value);
    let kind = js_value_to_u8(array.get(0))?;

    match kind {
        ACTION_STAY => Some(AgentAction::Stay),
        ACTION_MOVE => Some(AgentAction::Move(point_from_action_array(&array)?)),
        ACTION_TAG => Some(AgentAction::Tag(point_from_action_array(&array)?)),
        _ => None,
    }
}

fn point_from_action_array(array: &js_sys::Array) -> Option<Point2D> {
    Some(Point2D {
        x: js_value_to_usize(array.get(1))?,
        y: js_value_to_usize(array.get(2))?,
    })
}

fn js_value_to_u8(value: JsValue) -> Option<u8> {
    let number = value.as_f64()?;

    if !number.is_finite() || number.fract() != 0.0 || number < 0.0 || number > u8::MAX as f64 {
        return None;
    }

    Some(number as u8)
}

fn js_value_to_usize(value: JsValue) -> Option<usize> {
    let number = value.as_f64()?;

    if !number.is_finite() || number.fract() != 0.0 || number < 0.0 {
        return None;
    }

    Some(number as usize)
}

#[wasm_bindgen(js_name = stayAction)]
pub fn stay_action() -> js_sys::Array {
    action_array(ACTION_STAY, None)
}

#[wasm_bindgen(js_name = moveAction)]
pub fn move_action(x: usize, y: usize) -> js_sys::Array {
    action_array(ACTION_MOVE, Some(Point2D { x, y }))
}

#[wasm_bindgen(js_name = tagAction)]
pub fn tag_action(x: usize, y: usize) -> js_sys::Array {
    action_array(ACTION_TAG, Some(Point2D { x, y }))
}

fn action_array(kind: u8, point: Option<Point2D>) -> js_sys::Array {
    let action = js_sys::Array::new();
    action.push(&JsValue::from_f64(kind as f64));

    if let Some(point) = point {
        action.push(&JsValue::from_f64(point.x as f64));
        action.push(&JsValue::from_f64(point.y as f64));
    }

    action
}
