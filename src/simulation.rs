use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Point2D {
    pub x: u32,
    pub y: u32,
}

pub struct AgentInput {
    pub position: Point2D,
    pub can_tag: bool,
}

pub enum AgentAction {
    Move(Point2D),
    Tag,
    Stay,
}

pub trait Agent {
    fn act(&self, input: AgentInput) -> AgentAction;
}

pub struct Simulation {
    rows: u32,
    cols: u32,
    // probably not most effective
    grid: HashMap<Point2D, Box<dyn Agent>>,
}

impl Simulation {
    pub fn new(rows: u32, cols: u32) -> Self {
        Self {
            rows,
            cols,
            grid: HashMap::new(),
        }
    }

    pub fn place_agent(&mut self, agent: Box<dyn Agent>, x: u32, y: u32) {
        let point = Point2D { x, y };
        // Note: does not handle overlap when placing agents
        // which can become a problem as this method is designed to be
        // called by external code that does not know the grid state.
        self.grid.entry(point).insert_entry(agent);
    }

    fn apply_action(&mut self, point: Point2D, action: AgentAction) {
        match action {
            AgentAction::Move(move_to) => {
                let agent = self.grid.remove(&point).unwrap();
                self.grid.insert(move_to, agent);
            }
            AgentAction::Tag => todo!(),
            AgentAction::Stay => (),
        }
    }

    fn tick(&mut self) {
        let mut actions: Vec<(Point2D, AgentAction)> = Vec::new();
        for (point, agent) in self.grid.iter_mut() {
            let input = AgentInput {
                position: point.clone(),
                can_tag: false,
            };
            let action = agent.act(input);
            actions.push((point.clone(), action))
        }
        for (point, action) in actions {
            self.apply_action(point, action);
        }
    }

    fn show_grid(&self) {
        println!("┌{}┐", "─".repeat(self.cols as usize));
        for y in 0..self.rows {
            for x in 0..self.cols {
                let point = Point2D { x, y };
                if self.grid.contains_key(&point) {
                    print!("x");
                } else {
                    print!(".");
                }
            }
            println!();
        }
        println!("└{}┘", "─".repeat(self.cols as usize));
    }

    pub fn run(&mut self) {
        // TODO: pick a random agent to be tagged
        // also: track who is currently it and who was the
        // previous tagger as to not swap.
        // counter is just for debugging
        let mut counter = 0;
        loop {
            if counter == 5 {
                break;
            }
            self.tick();
            self.show_grid();
            counter += 1;
        }
    }
}
