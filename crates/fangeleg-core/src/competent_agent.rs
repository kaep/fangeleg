use crate::simulation::{Agent, AgentAction, AgentInput, CellState, Point2D};

pub struct CompetentAgent;

impl Agent for CompetentAgent {
    fn act(
        &self,
        AgentInput {
            position,
            taggable_positions,
            grid_view,
            is_tagger,
        }: AgentInput,
    ) -> AgentAction {
        // If we are the tagger and have targets in range, grab the first
        if is_tagger && let Some(target) = taggable_positions.first() {
            return AgentAction::Tag(*target);
        }

        let possible_targets = find_possible_targets(&grid_view);
        let valid_moves = find_valid_moves(&grid_view, position);

        if is_tagger {
            // If we are the tagger and have no taggable positions, chase nearest possible target
            if let Some(target) = find_nearest_target(&possible_targets, position) {
                // Find the best move towards the target
                if let Some(best_move_towards_target) = valid_moves
                    .iter()
                    .copied()
                    .min_by_key(|valid_move| manhattan_distance(*valid_move, target))
                {
                    return AgentAction::Move(best_move_towards_target);
                } else {
                    // No best move towards target exist, stay
                    return AgentAction::Stay;
                }
            }
        } else if let Some(tagger_position) = find_tagger_position(&grid_view) {
            // Find the best move away from the tagger
            // TODO: repeat of logic above, abstracted into a helper function?
            if let Some(best_move_away_from_tagger) = valid_moves
                .iter()
                .copied()
                .max_by_key(|valid_move| manhattan_distance(*valid_move, tagger_position))
            {
                return AgentAction::Move(best_move_away_from_tagger);
            } else {
                // No best move away from tagger exist, stay
                return AgentAction::Stay;
            }
        }

        // No valid moves exist, stay
        AgentAction::Stay
    }
}

fn find_valid_moves(grid_view: &[Vec<CellState>], Point2D { x, y }: Point2D) -> Vec<Point2D> {
    let neighbors = [
        x.checked_sub(1).map(|x| Point2D { x, y }),
        x.checked_add(1).map(|x| Point2D { x, y }),
        y.checked_sub(1).map(|y| Point2D { x, y }),
        y.checked_add(1).map(|y| Point2D { x, y }),
    ];

    // This could panic if the simulation grid is empty
    let row_count = grid_view.len();
    let col_count = grid_view[0].len();

    neighbors
        .into_iter()
        .flatten()
        // Filter out positions that are out of bounds
        .filter(|point| point.x < col_count && point.y < row_count)
        // Filter out the positions that are occupied by other agents
        .filter(|point| grid_view[point.y][point.x] == CellState::Empty)
        .collect()
}

fn manhattan_distance(a: Point2D, b: Point2D) -> usize {
    a.x.abs_diff(b.x) + a.y.abs_diff(b.y)
}

fn find_possible_targets(grid_view: &[Vec<CellState>]) -> Vec<Point2D> {
    let mut positions = Vec::new();
    for (y, row) in grid_view.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            if *cell == CellState::Agent {
                positions.push(Point2D { x, y });
            }
        }
    }

    positions
}

fn find_nearest_target(possible_targets: &[Point2D], position: Point2D) -> Option<Point2D> {
    possible_targets
        .iter()
        .copied()
        .min_by_key(|target_position| manhattan_distance(position, *target_position))
}

// TODO: duplication
// the tagger position could be found while scanning the grid for possible targets,
// or better: passed from sim as it already has to create the grid view
fn find_tagger_position(grid_view: &[Vec<CellState>]) -> Option<Point2D> {
    let mut tagger_position = None;
    for (y, row) in grid_view.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            if *cell == CellState::Tagger {
                tagger_position = Some(Point2D { x, y });
            }
        }
    }

    tagger_position
}
