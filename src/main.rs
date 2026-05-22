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
            let x = rand::random_range(0..cols);
            let y = rand::random_range(0..rows);
            if sim.place_agent(Box::new(NaiveAgent {}), x, y).is_ok() {
                break;
            }
        }
    }

    sim.run_iterations(5);
}
