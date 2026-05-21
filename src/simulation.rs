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

pub struct AgentId(u32);

pub struct AgentEntry {
    pub id: AgentId,
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

    fn next_agent_id(&mut self) -> AgentId {
        let id = AgentId(self.next_agent_id);
        // Note: Can technically overflow
        self.next_agent_id += 1;
        id
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

        if self.grid[y][x].is_some() {
            return Err(SimulationError::CellOccupied(point));
        }

        self.grid[y][x] = Some(AgentEntry {
            id: self.next_agent_id(),
            agent,
        });

        Ok(())
    }

    fn apply_action(
        &mut self,
        Point2D { x, y }: Point2D,
        action: AgentAction,
    ) -> Result<(), SimulationError> {
        match action {
            // TODO: match struct instead of dotting into fields?
            AgentAction::Move(move_to) => {
                // Bounds check the new position
                if move_to.x >= self.cols || move_to.y >= self.rows {
                    return Err(SimulationError::OutOfBounds(move_to));
                }

                // Trying to move to the source position is a no-op
                if move_to.x == x && move_to.y == y {
                    return Ok(());
                }

                // Trying to move to an occupied position should fail
                if self.grid[move_to.y][move_to.x].is_some() {
                    return Err(SimulationError::CellOccupied(move_to));
                }

                // Take out the agent from the current position
                let agent = self.grid[y][x].take();
                // New position is valid: move the agent to it
                self.grid[move_to.y][move_to.x] = agent;
                Ok(())
            }
            AgentAction::Tag => todo!(),
            AgentAction::Stay => Ok(()),
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
                    let _ = self.apply_action(point, action);
                }
            }
        }
    }

    fn show_grid(&self) {
        println!("┌{}┐", "─".repeat(self.cols));
        for y in 0..self.rows {
            for x in 0..self.cols {
                if self.grid[y][x].is_some() {
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
        loop {
            self.tick();
            self.show_grid();
        }
    }

    pub fn run_iterations(&mut self, iterations: usize) {
        for _ in 0..iterations {
            self.tick();
            self.show_grid();
        }
    }
}
