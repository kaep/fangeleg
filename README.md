# Fangeleg

![Fangeleg logo](logo.png)

Fangeleg is a small simulator for an agent-based model where agents play the game of Tag, called "fangeleg" in Danish.

Agents are simulated as autonomous entities that move around a 2D grid and tag each other. At each step of the simulation, agents can move, tag, or remain stationary. Agent behavior is defined via  trait, which allows for custom agent implementations.

Currently two agent types are supported: `NaiveAgent` and `CompetentAgent`. Naive agents are simple and do not use any advanced strategies, while competent agents are more sophisticated and employ strategies like chasing nearest target and avoiding the one who is "it".
Simulating with only competent agents quickly converges to a "deadlock"-ish state where agents just dance around without ever tagging each other.

The project also includes a web-based visualization of the simulation running in WebAssembly and an API for defining custom agent behaviors using JavaScript.

## Usage
`just run` to build the project and serve it locally. View the simulation in your browser at `http://localhost:8080`.
Explore `justfile` for more commands.

Unit tests exist for simulator logic like agent movement and tagging. Run them with `cargo test`.

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

## Limitations
The simulator currently ticks and processes grid positions sequentially. This means that agents may act multiple times per tick, if they are processed and make a move that puts them in a position that has not yet been processed.

## Possible future directions

### Analysis tools
Agent behavior is already abstracted with a trait, so experimentation with different strategies is straightforward.
If the simulator engine is extended to track agent behavior, it is possible to analyze which strategies are most effective and why.
