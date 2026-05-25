use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::js_agent::JsAgent;
use fangeleg_core::competent_agent::CompetentAgent;
use fangeleg_core::naive_agent::NaiveAgent;
use fangeleg_core::simulation::{CellState, Simulation};

const COLOR_BACKGROUND: &str = "#111827";
const COLOR_GRID: &str = "#374151";
const COLOR_AGENT: &str = "#60a5fa";
const COLOR_CURRENT_TAGGER: &str = "#ef4444";
const COLOR_TAGBACK_IMMUNE: &str = "#f59e0b";
const COLOR_TEXT: &str = "#e5e7eb";

const LEGEND_HEIGHT: f64 = 48.0;

#[wasm_bindgen]
pub struct WasmSimulation {
    simulation: Simulation,
    context: CanvasRenderingContext2d,
    cell_size: f64,
}

#[wasm_bindgen]
impl WasmSimulation {
    #[wasm_bindgen(constructor)]
    pub fn new(
        canvas_id: &str,
        rows: usize,
        cols: usize,
        num_agents: usize,
        cell_size: f64,
    ) -> Result<WasmSimulation, JsValue> {
        let window = web_sys::window().ok_or("missing window")?;
        let document = window.document().ok_or("missing document")?;

        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or("canvas not found")?
            .dyn_into::<HtmlCanvasElement>()?;

        canvas.set_width((cols as f64 * cell_size) as u32);
        canvas.set_height((rows as f64 * cell_size + LEGEND_HEIGHT) as u32);

        let context = canvas
            .get_context("2d")?
            .ok_or("2d context not available")?
            .dyn_into::<CanvasRenderingContext2d>()?;

        let mut simulation = Simulation::new(rows, cols);

        let max_agents = rows * cols;
        let num_agents = num_agents.min(max_agents);

        for _ in 0..num_agents {
            loop {
                let x = rand::random_range(0..cols);
                let y = rand::random_range(0..rows);

                // Coin flip to determine whether agent should be competent or naive
                let is_competent = rand::random::<bool>();
                if is_competent {
                    if simulation
                        .place_agent(Box::new(CompetentAgent {}), x, y)
                        .is_ok()
                    {
                        break;
                    }
                } else {
                    if simulation
                        .place_agent(Box::new(NaiveAgent {}), x, y)
                        .is_ok()
                    {
                        break;
                    }
                }
            }
        }

        simulation.ensure_tagger_chosen();

        let sim = WasmSimulation {
            simulation,
            context,
            cell_size,
        };

        sim.draw();

        Ok(sim)
    }

    pub fn step(&mut self) {
        self.simulation.step();
        self.draw();
    }

    pub fn draw(&self) {
        self.clear();
        self.draw_grid();
        self.draw_agents();
        self.draw_legend();
    }

    pub fn place_js_agent(&mut self, x: usize, y: usize, act_function: js_sys::Function) -> bool {
        let placed = self
            .simulation
            .place_agent(Box::new(JsAgent::new(act_function)), x, y)
            .is_ok();

        if placed {
            self.simulation.ensure_tagger_chosen();
            self.draw();
        }

        placed
    }
}

impl WasmSimulation {
    fn clear(&self) {
        let width = self.simulation.cols() as f64 * self.cell_size;
        let height = self.simulation.rows() as f64 * self.cell_size + LEGEND_HEIGHT;

        self.context.set_fill_style_str(COLOR_BACKGROUND);
        self.context.fill_rect(0.0, 0.0, width, height);
    }

    fn draw_grid(&self) {
        self.context.set_stroke_style_str(COLOR_GRID);
        self.context.set_line_width(1.0);

        let width = self.simulation.cols() as f64 * self.cell_size;
        let height = self.simulation.rows() as f64 * self.cell_size;

        for x in 0..=self.simulation.cols() {
            let px = x as f64 * self.cell_size;
            self.context.begin_path();
            self.context.move_to(px, 0.0);
            self.context.line_to(px, height);
            self.context.stroke();
        }

        for y in 0..=self.simulation.rows() {
            let py = y as f64 * self.cell_size;
            self.context.begin_path();
            self.context.move_to(0.0, py);
            self.context.line_to(width, py);
            self.context.stroke();
        }
    }

    fn draw_agents(&self) {
        for y in 0..self.simulation.rows() {
            for x in 0..self.simulation.cols() {
                let Some(cell_state) = self.simulation.cell_state(x, y) else {
                    continue;
                };

                let color = match cell_state {
                    CellState::Empty => continue,
                    CellState::Agent => COLOR_AGENT,
                    CellState::Tagger => COLOR_CURRENT_TAGGER,
                    CellState::TagbackImmune => COLOR_TAGBACK_IMMUNE,
                };

                self.draw_agent(x, y, color);
            }
        }
    }

    fn draw_agent(&self, x: usize, y: usize, color: &str) {
        let padding = self.cell_size * 0.15;
        let size = self.cell_size - padding * 2.0;

        let px = x as f64 * self.cell_size + padding;
        let py = y as f64 * self.cell_size + padding;

        self.context.set_fill_style_str(color);
        self.context.begin_path();

        let radius = size / 2.0;
        let cx = px + radius;
        let cy = py + radius;

        self.context
            .arc(cx, cy, radius, 0.0, std::f64::consts::PI * 2.0)
            .unwrap();

        self.context.fill();
    }

    fn draw_legend(&self) {
        let grid_height = self.simulation.rows() as f64 * self.cell_size;
        let y = grid_height + 24.0;

        self.context.set_font("14px system-ui, sans-serif");
        self.context.set_fill_style_str(COLOR_TEXT);
        self.context
            .fill_text("Legend:", 8.0, y)
            .expect("failed to draw legend title");

        let mut x = 76.0;

        x = self.draw_legend_item(x, y, COLOR_AGENT, "Agent");
        x = self.draw_legend_item(x, y, COLOR_CURRENT_TAGGER, "It");
        self.draw_legend_item(x, y, COLOR_TAGBACK_IMMUNE, "Immune");
    }

    fn draw_legend_item(&self, x: f64, y: f64, color: &str, label: &str) -> f64 {
        let marker_radius = 6.0;
        let marker_x = x + marker_radius;
        let marker_y = y - marker_radius + 1.0;

        self.context.set_fill_style_str(color);
        self.context.begin_path();
        self.context
            .arc(
                marker_x,
                marker_y,
                marker_radius,
                0.0,
                std::f64::consts::PI * 2.0,
            )
            .expect("failed to draw legend marker");
        self.context.fill();

        let text_x = x + marker_radius * 2.0 + 6.0;
        self.context.set_fill_style_str(COLOR_TEXT);
        self.context
            .fill_text(label, text_x, y)
            .expect("failed to draw legend label");

        text_x + label.len() as f64 * 8.0 + 24.0
    }
}
