# Fangeleg

![Fangeleg logo](logo.png)

Fangeleg is a small simulator for an agent-based model where agents play the game of Tag, called "fangeleg" in Danish.

Agents are simulated as autonomous entities that move around a 2D grid and tag each other. At each step of the simulation, agents can move, tag, or remain stationary. Agent behavior is defined via an `Agent` trait, which allows for custom agent implementations.

Currently two built-in agent types are supported: `NaiveAgent` and `CompetentAgent`. Naive agents are simple and do not use any advanced strategies, while competent agents are more sophisticated and employ strategies like chasing nearest target and avoiding the one who is "it".
Simulating with only competent agents quickly converges to a "deadlock"-ish state where agents just dance around without ever tagging each other.

The project also includes a web-based visualization of the simulation running in WebAssembly and an API for defining custom agent behaviors using JavaScript. See `web/index.html` for an example of usage and `crates/fangeleg-wasm/src/js_agent.rs` for the Rust-side implementation.

## Game rules
One agent is "it" and the goal is to tag the other agents. When an agent is tagged, it becomes the new tagger. The previous tagger becomes temporarily immune, preventing immediate tag-backs.

## Usage
`just run` to build the project and serve the frontend locally. View the simulation in your browser at `http://localhost:8080`.

Currently this starts a 30x30 simulation with 40 agents that are 50/50 split between competent and naive agents by default.
The web UI provides buttons for choosing between 100% naive agents, a 50/50 split, or 100% competent agents. Grid size, tick speed, and agent count need to be configured in `web/index.html`.
A fourth button places a JavaScript-defined agent at (0, 0), but this can fail if the cell is occupied by another agent.

Explore `justfile` for more commands related to building, cleaning, and running the simulation.

Run a simple terminal version with `cargo run -p fangeleg-cli`.

Unit tests exist for simulator logic like agent movement and tagging. Run them with `cargo test --workspace`.

## Requirements

With Nix flakes run:
```sh
nix develop
```

If using direnv with Nix flakes run:
```sh
direnv allow
```

Otherwise you need
- Rust toolchain (e.g. via `rustup`) with support for the `wasm32-unknown-unknown` target
- `wasm-bindgen` CLI v0.2.108
- `miniserve`
- `just` CLI (for convenience)

## Project structure
The project is organized into three crates:
- `fangeleg-core` contains the main simulator logic, agent trait definition and agent implementations
- `fangeleg-wasm` has the WASM-based web visualization
- `fangeleg-cli` provides a simple terminal demo of the simulator

## Limitations
The simulator currently resolves ticks sequentially. Agents act at most once per tick, but each agent observes the grid state as it exists when its turn is processed, so earlier actions in the same tick can affect later decisions.

Due to limitations of `wasm-bindgen`, the `AgentAction` enum is represented as an array on the JavaScript side. Helper functions are exposed to help with this.

The web visualization currently exposes controls for choosing the agent type mix, but grid size, tick speed, and agent count are still configured in `web/index.html`. A future improvement would be to make those values configurable directly in the browser as well. The button for placing a JavaScript-defined agent at (0, 0) can also currently fail if the cell is occupied by another agent.

## Possible future directions

### Analysis tools
Agent behavior is already abstracted with a trait, so experimentation with different strategies is straightforward.
If the simulator engine is extended to track agent behavior, it is possible to analyze which strategies are most effective and why.

### 1D vector representation of grid state
The grid state is currently represented as a nested vector that is indexed by `[row][col]`/`[y][x]`.
This requires knowledge about how to correctly index and also allocation of separate vectors for each row.
A flat vector representation could be used instead. This would require fewer memory allocations and could be used in combination with a simple indexing API (`index = y * cols + x`) to reduce potential for indexing errors.

### Simulation module organization
The `simulation` module is quite large and the `impl Simulation` block mixes public API and private helpers.
Introducing submodules or even just restructuring the `impl` block could improve readability.
