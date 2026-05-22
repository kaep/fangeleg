use rand::random_range;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Point2D {
    pub x: usize,
    pub y: usize,
}

pub struct AgentInput {
    pub position: Point2D,
    pub taggable_positions: Vec<Point2D>,
}

pub enum AgentAction {
    Move(Point2D),
    Tag(Point2D),
    Stay,
}

pub trait Agent {
    fn act(&self, input: AgentInput) -> AgentAction;
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
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

#[derive(Debug)]
pub enum SimulationError {
    CellOccupied(Point2D),
    OutOfBounds(Point2D),
}

pub struct Simulation {
    rows: usize,
    cols: usize,
    grid: Vec<Vec<Option<AgentEntry>>>,
    current_tagger: Option<AgentId>,
    tagback_immune_agent: Option<AgentId>,
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
            current_tagger: None,
            tagback_immune_agent: None,
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
            AgentAction::Tag(target_position) => {
                // At this point the simulator has already made sure that
                // the agent at the target position is taggable and that
                // the current agent is the tagger, but validate the
                // decision made by the agent anyways as it technically
                // cannot be trusted.
                // Errors detected are ignored with Ok(()) as it seems
                // harsh to fail the sim given an attempt to do something illegal.
                let Some(tagger) = self.grid[y][x].as_ref() else {
                    return Ok(());
                };

                let tagger_id = tagger.id;
                let tagger_position = Point2D { x, y };

                // Actually redundant because the later call to find_taggable_positions
                // returns [] if the agent is not the tagger
                if self.current_tagger != Some(tagger_id) {
                    // Should the sim fail? ignore the error?
                    return Ok(());
                }

                if !self
                    .find_taggable_positions(tagger_id, tagger_position)
                    .contains(&target_position)
                {
                    return Ok(());
                }

                let Some(tagged) = self.grid[target_position.y][target_position.x].as_ref() else {
                    return Ok(());
                };

                // The tagger becomes immune for the next round
                self.tagback_immune_agent = Some(tagger.id);

                // The target becomes the tagger for the next round
                self.current_tagger = Some(tagged.id);

                Ok(())
            }
            AgentAction::Stay => Ok(()),
        }
    }

    fn taggable_adjacent_positions(&self, Point2D { x, y }: Point2D) -> Vec<Point2D> {
        let neighbors = [
            x.checked_sub(1).map(|x| Point2D { x, y }),
            x.checked_add(1).map(|x| Point2D { x, y }),
            y.checked_sub(1).map(|y| Point2D { x, y }),
            y.checked_add(1).map(|y| Point2D { x, y }),
        ];

        neighbors
            .into_iter()
            .flatten()
            // Filter out positions that are out of bounds
            .filter(|point| point.x < self.cols && point.y < self.rows)
            // Filter out the position that is immune to the current tagger
            .filter(|point| {
                self.grid[point.y][point.x]
                    .as_ref()
                    .is_some_and(|agent| Some(agent.id) != self.tagback_immune_agent)
            })
            .collect()
    }

    fn can_agent_tag(&self, agent_id: AgentId) -> bool {
        self.current_tagger == Some(agent_id)
    }

    fn find_taggable_positions(&self, agent_id: AgentId, position: Point2D) -> Vec<Point2D> {
        if !self.can_agent_tag(agent_id) {
            return vec![];
        }

        self.taggable_adjacent_positions(position)
    }

    fn tick(&mut self) {
        for y in 0..self.rows {
            for x in 0..self.cols {
                let point = Point2D { x, y };
                if let Some(agent) = self.grid[y][x].as_ref() {
                    let input = AgentInput {
                        // Is cloning the point cheaper than creating a new one?
                        position: point,
                        taggable_positions: self.find_taggable_positions(agent.id, point),
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
                    // Unwrap is safe given the check above
                    let agent_id = self.grid[y][x].as_ref().unwrap().id;
                    if Some(agent_id) == self.current_tagger {
                        print!("T");
                        continue;
                    }
                    print!("x");
                } else {
                    print!(".");
                }
            }
            println!();
        }
        println!(
            "Current tagger: {:?}",
            self.current_tagger
                .map_or_else(|| "none".to_string(), |agent_id| agent_id.0.to_string())
        );
        println!("└{}┘", "─".repeat(self.cols));
    }

    fn choose_random_tagger(&mut self) {
        // Avoid panicking by not choosing a tagger if there are no agents
        if self.next_agent_id == 0 {
            return;
        }
        let random_agent = random_range(0..self.next_agent_id);
        self.current_tagger = Some(AgentId(random_agent));
    }

    // TODO: reduce duplication in run methods
    pub fn run(&mut self) {
        // Choose a random tagger to start with
        // This might make sense to expose to callers at some point.
        self.choose_random_tagger();

        loop {
            self.tick();
            self.show_grid();
        }
    }

    pub fn run_iterations(&mut self, iterations: usize) {
        // Choose a random tagger to start with
        // This might make sense to expose to callers at some point.
        self.choose_random_tagger();
        for _ in 0..iterations {
            self.tick();
            self.show_grid();
        }
    }
}
