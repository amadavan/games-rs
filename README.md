# games-rs

[![Rust](https://github.com/amadavan/games-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/amadavan/games-rs/actions/workflows/rust.yml)

A collection of classic board games implemented in Rust, designed for building and testing AI agents.

## Overview

This project implements various board games with a unified interface (`GameBoard` trait), making it easy to develop, test, and compare different AI strategies. The framework supports both human players and sophisticated AI agents, including Monte Carlo Graph Search with UCT (Upper Confidence bounds applied to Trees).

**Key Features:**
- Generic game interface for consistent agent development
- Multiple AI agent types (random, Monte Carlo, human)
- Persistent learning through graph serialization
- Type-safe move validation and game state management

## Games

### Ultimate Tic-Tac-Toe
A complex variant of tic-tac-toe where the board consists of a 3×3 grid of smaller tic-tac-toe boards. Players must win individual boards to claim positions in the outer board, with the twist that each move determines which board the opponent must play in next.

**Move Format:** Four space-separated numbers: `microboard_row microboard_col cell_row cell_col`

**Rules:**
- Win three microboards in a row to win the game
- Your move determines which board your opponent plays on next
- If directed to a completed board, play anywhere

### Connect Four
The classic vertical grid game where players drop tokens into columns, aiming to connect four in a row horizontally, vertically, or diagonally.

**Move Format:** Column number (0-6)

**Rules:**
- 6 rows × 7 columns grid
- Tokens fall to the lowest available position
- First to connect four wins

## AI Agents

The framework provides several agent types that implement the `Agent<Game>` trait:

### PlayerAgent
Interactive human player via terminal input. Displays the current board state and prompts for moves.

```rust
let agent = PlayerAgent::new(1); // Player 1
```

### RandomAgent
Selects moves uniformly at random from available moves. Useful as a baseline for comparison.

```rust
let agent = RandomAgent::new();
```

### MonteCarloGraphSearch
Advanced AI using Monte Carlo Graph Search with UCT. Maintains a graph of game states and learns from game outcomes to make progressively better decisions.

**Features:**
- Balances exploration vs exploitation using the UCT formula: `w/n + √(2·ln(N)/n)`
- Learns from every game played via backpropagation
- Supports saving/loading trained graphs for persistent learning

```rust
// Create a new agent
let mut agent = MonteCarloGraphSearch::new();

// Or load a pre-trained graph
let graph = MonteCarloGraph::from_file("trained_model.bin")?;
let agent = MonteCarloGraphSearch::from_graph(graph);
```

### Custom Agents
Implement the `Agent<Game>` trait to create your own AI:

```rust
pub trait Agent<Game: GameBoard> {
    fn get_move(&self, board: &Game) -> Game::MoveType;

    fn notify(&mut self, moves: &Vec<(u8, Game)>, status: BoardStatus) {
        // Optional: learn from game outcomes
    }
}
```

## Running Games

Play games using the unified play binary:

```bash
# Play Ultimate Tic-Tac-Toe
cargo run --bin play -- --game ultimate_ttt

# Play Connect Four
cargo run --bin play -- --game connect_four
```

### Training AI Agents

Run multiple games to train Monte Carlo agents:

```bash
# Train an agent by playing games
cargo run --bin play -- --game connect_four --train --iterations 1000
```

The trained graph will be saved and can be loaded for future games.

## Project Structure

```
games-rs/
├── src/
│   ├── lib.rs              # Core GameBoard trait and game engine
│   ├── ultimate_ttt.rs     # Ultimate Tic-Tac-Toe implementation
│   ├── connect_four.rs     # Connect Four implementation
│   ├── agents/
│   │   ├── mod.rs                 # Agent trait and implementations
│   │   └── monte_carlo_graph.rs   # Monte Carlo graph search data structure
│   ├── common/
│   │   ├── mod.rs          # Shared utilities
│   │   └── filesystem.rs   # File I/O helpers
│   └── bin/
│       └── play.rs         # Game executable
└── macros/                 # Procedural macros for game development
```

## Building Your Own Agent

To create a custom AI agent, implement the `Agent<Game>` trait:

```rust
use games_rs::{GameBoard, BoardStatus, agents::Agent};
use games_rs::connect_four::ConnectFour;

struct MyAgent {
    // Your agent's state
}

impl Agent<ConnectFour> for MyAgent {
    fn get_move(&self, board: &ConnectFour) -> usize {
        // Implement your strategy here
        // - Minimax with alpha-beta pruning
        // - Neural network evaluation
        // - Reinforcement learning
        // - Or any other approach!

        let moves = board.get_available_moves();
        // ... your logic ...
        moves[0]
    }

    fn notify(&mut self, moves: &Vec<(u8, ConnectFour)>, status: BoardStatus) {
        // Optional: Update your agent based on the game outcome
        // Useful for learning algorithms
    }
}
```

### Using the GameBoard Trait

All games implement the `GameBoard` trait, providing a consistent interface:

```rust
pub trait GameBoard {
    type MoveType;

    fn get_current_player(&self) -> u8;
    fn get_available_moves(&self) -> Vec<Self::MoveType>;
    fn play(&mut self, mv: Self::MoveType, player: impl Into<u8>) -> Result<(), String>;
    fn get_status(&self) -> BoardStatus;
}
```

This allows you to write generic algorithms that work with any game!

## Development

### Prerequisites
- Rust 2024 edition or later
- Cargo (comes with Rust)

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release
```

### Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_back_propogate
```

### Documentation

Generate and view the documentation:

```bash
cargo doc --open
```

## Examples

### Playing a Game Between Two AIs

```rust
use games_rs::{play_game, BoardStatus};
use games_rs::connect_four::ConnectFour;
use games_rs::agents::{RandomAgent, MonteCarloGraphSearch};

fn main() {
    let agent1 = RandomAgent::new();
    let agent2 = MonteCarloGraphSearch::<ConnectFour>::new();

    let result = play_game::<ConnectFour, _, _>(&agent1, &agent2);

    match result {
        BoardStatus::Win(player) => println!("Player {} wins!", player),
        BoardStatus::Draw => println!("It's a draw!"),
        BoardStatus::InProgress => println!("Game in progress"),
    }
}
```

### Training a Monte Carlo Agent

```rust
use games_rs::{play_game, BoardStatus};
use games_rs::connect_four::ConnectFour;
use games_rs::agents::{Agent, RandomAgent, MonteCarloGraphSearch};

fn main() {
    let mut agent = MonteCarloGraphSearch::<ConnectFour>::new();
    let opponent = RandomAgent::new();

    // Train for 10,000 games
    for i in 0..10_000 {
        let mut game = ConnectFour::default();
        let mut history = Vec::new();

        loop {
            let current_player = game.get_current_player();
            let mv = if current_player == 1 {
                agent.get_move(&game)
            } else {
                opponent.get_move(&game)
            };

            game.play(mv, current_player).unwrap();
            history.push((current_player, game));

            if game.get_status() != BoardStatus::InProgress {
                agent.notify(&history, game.get_status());
                break;
            }
        }

        if i % 1000 == 0 {
            println!("Completed {} games", i);
        }
    }

    // Save the trained agent
    // agent.graph.to_file("trained_agent.bin").unwrap();
}
```

## Contributing

Contributions are welcome! Areas of interest:
- New game implementations
- Additional AI agents (minimax, neural networks, etc.)
- Performance optimizations
- Documentation improvements

## License

This project is open source and available under the MIT License.
