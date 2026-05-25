use std::collections::HashSet;

use rand::random_range;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Point2D {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug)]
pub struct AgentInput {
    pub position: Point2D,
    pub taggable_positions: Vec<Point2D>,
    pub grid_view: Vec<Vec<CellState>>,
    pub is_tagger: bool,
}

#[derive(Clone, Copy)]
pub enum AgentAction {
    Move(Point2D),
    Tag(Point2D),
    Stay,
}

pub trait Agent {
    fn act(&self, input: AgentInput) -> AgentAction;
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellState {
    Empty,
    Agent,
    Tagger,
    TagbackImmune,
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
        let mut already_acted = HashSet::new();
        for y in 0..self.rows {
            for x in 0..self.cols {
                let point = Point2D { x, y };
                if let Some(agent) = self.grid[y][x].as_ref() {
                    if !already_acted.insert(agent.id) {
                        continue;
                    }
                    let input = AgentInput {
                        // Is cloning the point cheaper than creating a new one?
                        position: point,
                        taggable_positions: self.find_taggable_positions(agent.id, point),
                        grid_view: self.create_grid_view(),
                        is_tagger: self.current_tagger == Some(agent.id),
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
        self.ensure_tagger_chosen();

        loop {
            self.tick();
            self.show_grid();
        }
    }

    pub fn run_iterations(&mut self, iterations: usize) {
        self.ensure_tagger_chosen();
        for _ in 0..iterations {
            self.tick();
            self.show_grid();
        }
    }

    pub fn step(&mut self) {
        self.tick();
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn cell_state(&self, x: usize, y: usize) -> Option<CellState> {
        if x >= self.cols || y >= self.rows {
            return None;
        }

        let Some(agent) = self.grid[y][x].as_ref() else {
            return Some(CellState::Empty);
        };

        if Some(agent.id) == self.current_tagger {
            return Some(CellState::Tagger);
        }

        if Some(agent.id) == self.tagback_immune_agent {
            return Some(CellState::TagbackImmune);
        }

        Some(CellState::Agent)
    }

    pub fn ensure_tagger_chosen(&mut self) {
        if self.current_tagger.is_none() {
            self.choose_random_tagger();
        }
    }

    fn create_grid_view(&self) -> Vec<Vec<CellState>> {
        (0..self.rows)
            .map(|y| {
                (0..self.cols)
                    .map(|x| {
                        self.cell_state(x, y)
                            .expect("Out of bounds grid coordinate")
                    })
                    .collect()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestAgent;

    impl Agent for TestAgent {
        fn act(&self, _input: AgentInput) -> AgentAction {
            AgentAction::Stay
        }
    }

    fn test_agent() -> Box<dyn Agent> {
        Box::new(TestAgent)
    }

    fn agent_id_at(sim: &Simulation, x: usize, y: usize) -> AgentId {
        sim.grid[y][x]
            .as_ref()
            .expect("expected agent at position")
            .id
    }

    #[test]
    fn place_agent_returns_error_when_x_out_of_bounds() {
        let mut sim = Simulation::new(2, 3);

        let result = sim.place_agent(test_agent(), 3, 0);

        assert!(matches!(
            result,
            Err(SimulationError::OutOfBounds(Point2D { x: 3, y: 0 }))
        ));
    }

    #[test]
    fn place_agent_returns_error_when_y_out_of_bounds() {
        let mut sim = Simulation::new(2, 3);

        let result = sim.place_agent(test_agent(), 0, 2);

        assert!(matches!(
            result,
            Err(SimulationError::OutOfBounds(Point2D { x: 0, y: 2 }))
        ));
    }

    #[test]
    fn place_agent_returns_error_when_cell_is_occupied() {
        let mut sim = Simulation::new(2, 3);

        assert!(sim.place_agent(test_agent(), 1, 1).is_ok());

        let result = sim.place_agent(test_agent(), 1, 1);

        assert!(matches!(
            result,
            Err(SimulationError::CellOccupied(Point2D { x: 1, y: 1 }))
        ));
    }

    #[test]
    fn move_action_succeeds_when_destination_cell_is_empty() {
        let mut sim = Simulation::new(2, 3);

        sim.place_agent(test_agent(), 0, 0).unwrap();

        let result = sim.apply_action(
            Point2D { x: 0, y: 0 },
            AgentAction::Move(Point2D { x: 1, y: 0 }),
        );

        assert!(result.is_ok());
        assert!(sim.grid[0][0].is_none());
        assert!(sim.grid[0][1].is_some());
    }

    #[test]
    fn move_out_of_bounds_returns_error_and_keeps_agent_in_original_cell() {
        let mut sim = Simulation::new(2, 3);

        sim.place_agent(test_agent(), 2, 1).unwrap();

        let result = sim.apply_action(
            Point2D { x: 2, y: 1 },
            AgentAction::Move(Point2D { x: 3, y: 1 }),
        );

        assert!(matches!(
            result,
            Err(SimulationError::OutOfBounds(Point2D { x: 3, y: 1 }))
        ));
        assert!(sim.grid[1][2].is_some());
    }

    #[test]
    fn move_into_occupied_cell_fails_without_overwriting() {
        let mut sim = Simulation::new(2, 3);

        sim.place_agent(test_agent(), 0, 0).unwrap();
        sim.place_agent(test_agent(), 1, 0).unwrap();

        let original_target_id = agent_id_at(&sim, 1, 0);

        let result = sim.apply_action(
            Point2D { x: 0, y: 0 },
            AgentAction::Move(Point2D { x: 1, y: 0 }),
        );

        assert!(matches!(
            result,
            Err(SimulationError::CellOccupied(Point2D { x: 1, y: 0 }))
        ));
        assert!(sim.grid[0][0].is_some());
        assert_eq!(agent_id_at(&sim, 1, 0), original_target_id);
    }

    #[test]
    fn move_to_own_cell_is_noop() {
        let mut sim = Simulation::new(2, 3);

        sim.place_agent(test_agent(), 1, 1).unwrap();
        let original_id = agent_id_at(&sim, 1, 1);

        let result = sim.apply_action(
            Point2D { x: 1, y: 1 },
            AgentAction::Move(Point2D { x: 1, y: 1 }),
        );

        assert!(result.is_ok());
        assert_eq!(agent_id_at(&sim, 1, 1), original_id);
    }

    #[test]
    fn non_tagger_has_no_taggable_positions() {
        let mut sim = Simulation::new(2, 3);

        sim.place_agent(test_agent(), 0, 0).unwrap();
        sim.place_agent(test_agent(), 1, 0).unwrap();

        let non_tagger_id = agent_id_at(&sim, 0, 0);
        sim.current_tagger = Some(agent_id_at(&sim, 1, 0));

        let taggable_positions = sim.find_taggable_positions(non_tagger_id, Point2D { x: 0, y: 0 });

        assert!(taggable_positions.is_empty());
    }

    #[test]
    fn tagger_has_adjacent_occupied_position_as_taggable() {
        let mut sim = Simulation::new(2, 3);

        sim.place_agent(test_agent(), 0, 0).unwrap();
        sim.place_agent(test_agent(), 1, 0).unwrap();

        let tagger_id = agent_id_at(&sim, 0, 0);
        sim.current_tagger = Some(tagger_id);

        let taggable_positions = sim.find_taggable_positions(tagger_id, Point2D { x: 0, y: 0 });

        assert_eq!(taggable_positions, vec![Point2D { x: 1, y: 0 }]);
    }

    #[test]
    fn tagger_with_no_adjacent_occupied_positions_gets_no_taggable_positions() {
        let mut sim = Simulation::new(2, 3);

        sim.place_agent(test_agent(), 1, 1).unwrap();

        let tagger_id = agent_id_at(&sim, 1, 1);
        sim.current_tagger = Some(tagger_id);

        let taggable_positions = sim.find_taggable_positions(tagger_id, Point2D { x: 1, y: 1 });

        assert!(taggable_positions.is_empty());
    }

    #[test]
    fn tagger_does_not_get_adjacent_tagback_immune_agent_as_taggable() {
        let mut sim = Simulation::new(2, 3);

        sim.place_agent(test_agent(), 0, 0).unwrap();
        sim.place_agent(test_agent(), 1, 0).unwrap();

        let current_tagger_id = agent_id_at(&sim, 0, 0);
        let immune_id = agent_id_at(&sim, 1, 0);

        sim.current_tagger = Some(current_tagger_id);
        sim.tagback_immune_agent = Some(immune_id);

        let taggable_positions =
            sim.find_taggable_positions(current_tagger_id, Point2D { x: 0, y: 0 });

        assert!(taggable_positions.is_empty());
    }

    #[test]
    fn valid_tag_updates_current_tagger_to_target_agent() {
        let mut sim = Simulation::new(2, 3);

        sim.place_agent(test_agent(), 0, 0).unwrap();
        sim.place_agent(test_agent(), 1, 0).unwrap();

        let original_tagger_id = agent_id_at(&sim, 0, 0);
        let target_id = agent_id_at(&sim, 1, 0);

        sim.current_tagger = Some(original_tagger_id);

        let result = sim.apply_action(
            Point2D { x: 0, y: 0 },
            AgentAction::Tag(Point2D { x: 1, y: 0 }),
        );

        assert!(result.is_ok());
        assert_eq!(sim.current_tagger, Some(target_id));
    }

    #[test]
    fn valid_tag_makes_old_tagger_tagback_immune() {
        let mut sim = Simulation::new(2, 3);

        sim.place_agent(test_agent(), 0, 0).unwrap();
        sim.place_agent(test_agent(), 1, 0).unwrap();

        let original_tagger_id = agent_id_at(&sim, 0, 0);

        sim.current_tagger = Some(original_tagger_id);

        let result = sim.apply_action(
            Point2D { x: 0, y: 0 },
            AgentAction::Tag(Point2D { x: 1, y: 0 }),
        );

        assert!(result.is_ok());
        assert_eq!(sim.tagback_immune_agent, Some(original_tagger_id));
    }

    #[test]
    fn tag_from_non_tagger_is_ignored() {
        let mut sim = Simulation::new(2, 3);

        sim.place_agent(test_agent(), 0, 0).unwrap();
        sim.place_agent(test_agent(), 1, 0).unwrap();

        let non_tagger_id = agent_id_at(&sim, 0, 0);
        let actual_tagger_id = agent_id_at(&sim, 1, 0);

        sim.current_tagger = Some(actual_tagger_id);

        let result = sim.apply_action(
            Point2D { x: 0, y: 0 },
            AgentAction::Tag(Point2D { x: 1, y: 0 }),
        );

        assert!(result.is_ok());
        assert_eq!(sim.current_tagger, Some(actual_tagger_id));
        assert_eq!(sim.tagback_immune_agent, None);
        assert_ne!(sim.current_tagger, Some(non_tagger_id));
    }

    #[test]
    fn tag_to_non_adjacent_target_is_ignored() {
        let mut sim = Simulation::new(3, 3);

        sim.place_agent(test_agent(), 0, 0).unwrap();
        sim.place_agent(test_agent(), 2, 2).unwrap();

        let tagger_id = agent_id_at(&sim, 0, 0);
        let target_id = agent_id_at(&sim, 2, 2);

        sim.current_tagger = Some(tagger_id);

        let result = sim.apply_action(
            Point2D { x: 0, y: 0 },
            AgentAction::Tag(Point2D { x: 2, y: 2 }),
        );

        assert!(result.is_ok());
        assert_eq!(sim.current_tagger, Some(tagger_id));
        assert_ne!(sim.current_tagger, Some(target_id));
        assert_eq!(sim.tagback_immune_agent, None);
    }
}
