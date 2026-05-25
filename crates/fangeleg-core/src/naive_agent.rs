use rand::{random_bool, random_range};

use crate::simulation::{Agent, AgentAction, AgentInput, Point2D};

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct NaiveAgent;

impl Agent for NaiveAgent {
    fn act(
        &self,
        AgentInput {
            position,
            taggable_positions,
            // NaiveAgent does not care about this.
            // is_tagger is also implied by taggable_positions
            grid_view: _,
            is_tagger: _,
        }: AgentInput,
    ) -> AgentAction {
        if !taggable_positions.is_empty() {
            let wants_to_tag = random_bool(0.5);
            if wants_to_tag {
                let target = taggable_positions[random_range(0..taggable_positions.len())];
                return AgentAction::Tag(target);
            }
        }
        let wants_to_move = random_bool(0.5);
        if wants_to_move {
            let direction = self.decide_direction();
            let move_to = position.move_in_direction(direction);
            return AgentAction::Move(move_to);
        }

        AgentAction::Stay
    }
}

impl NaiveAgent {
    fn decide_direction(&self) -> Direction {
        let direction = random_range(0..4);
        match direction {
            0 => Direction::Up,
            1 => Direction::Down,
            2 => Direction::Left,
            3 => Direction::Right,
            _ => unreachable!(),
        }
    }
}

impl Point2D {
    fn move_in_direction(&self, direction: Direction) -> Self {
        match direction {
            Direction::Up => Self {
                x: self.x,
                y: self.y.saturating_sub(1),
            },
            Direction::Down => Self {
                x: self.x,
                y: self.y.saturating_add(1),
            },
            Direction::Left => Self {
                x: self.x.saturating_sub(1),
                y: self.y,
            },
            Direction::Right => Self {
                x: self.x.saturating_add(1),
                y: self.y,
            },
        }
    }
}
