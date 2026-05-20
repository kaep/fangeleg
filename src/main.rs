mod naive_agent;
pub mod simulation;

use naive_agent::NaiveAgent;
use simulation::Simulation;

fn main() {
    let rows = 30;
    let cols = 30;
    let mut sim = Simulation::new(rows, cols);

    let num_agents = 10;
    for _ in 0..num_agents {
        // Loop until all agents are placed successfully
        loop {
            let row = rand::random_range(0..rows);
            let col = rand::random_range(0..cols);
            if sim.place_agent(Box::new(NaiveAgent {}), row, col).is_ok() {
                break;
            }
        }
    }

    sim.run();
}
