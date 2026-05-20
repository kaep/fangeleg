#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Point2D {
    pub x: usize,
    pub y: usize,
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

pub struct AgentEntry {
    pub id: u32,
    pub agent: Box<dyn Agent>,
}

impl AgentEntry {
    pub fn act(&self, input: AgentInput) -> AgentAction {
        self.agent.act(input)
    }
}

pub enum SimulationError {
    CellOccupied(Point2D),
    OutOfBounds(Point2D),
}

pub struct Simulation {
    rows: usize,
    cols: usize,
    grid: Vec<Vec<Option<AgentEntry>>>,
    // Note: Can technically overflow
    next_agent_id: u32,
}

impl Simulation {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            // I dont love this but fill value requires clone on AgentEntry which i
            // dont like
            grid: (0..rows)
                .map(|_| (0..cols).map(|_| None).collect())
                .collect(),
            next_agent_id: 0,
        }
    }

    pub fn place_agent(
        &mut self,
        agent: Box<dyn Agent>,
        x: usize,
        y: usize,
    ) -> Result<(), SimulationError> {
        let point = Point2D { x, y };

        if x >= self.cols || y >= self.rows {
            return Err(SimulationError::OutOfBounds(point));
        }

        if let Some(_) = self.grid[y][x] {
            return Err(SimulationError::CellOccupied(point));
        }

        self.grid[y][x] = Some(AgentEntry {
            id: self.next_agent_id,
            agent,
        });

        // Note: Can technically overflow
        self.next_agent_id += 1;
        Ok(())
    }

    // TODO: return a result representing to represent out of bounds errors correctly
    fn apply_action(&mut self, Point2D { x, y }: Point2D, action: AgentAction) {
        match action {
            // TODO: match struct instead of dotting into fields?
            AgentAction::Move(move_to) => {
                // Bounds check the new position
                if move_to.x >= self.cols || move_to.y >= self.rows {
                    return;
                }

                // Take out the agent from the current position
                let agent = self.grid[y][x].take();
                // New position is valid: move the agent to it
                self.grid[move_to.y][move_to.x] = agent;
            }
            AgentAction::Tag => todo!(),
            AgentAction::Stay => (),
        }
    }

    fn tick(&mut self) {
        for y in 0..self.rows {
            for x in 0..self.cols {
                let point = Point2D { x, y };
                if let Some(agent) = self.grid[y][x].as_ref() {
                    let input = AgentInput {
                        // Is cloning the point cheaper than creating a new one?
                        position: point.clone(),
                        can_tag: false,
                    };
                    let action = agent.act(input);
                    self.apply_action(point, action);
                }
            }
        }
    }

    fn show_grid(&self) {
        println!("┌{}┐", "─".repeat(self.cols));
        for y in 0..self.rows {
            for x in 0..self.cols {
                if let Some(_) = self.grid[y][x] {
                    print!("x");
                } else {
                    print!(".");
                }
            }
            println!();
        }
        println!("└{}┘", "─".repeat(self.cols));
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
