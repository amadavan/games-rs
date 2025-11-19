# games-rs

[![Rust](https://github.com/amadavan/games-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/amadavan/games-rs/actions/workflows/rust.yml)

A collection of classic games implemented in Rust, designed for building and testing AI agents.

## Overview

This project implements various games with a focus on creating intelligent agents that can play them. Each game provides a common interface for agents to interact with, making it easy to develop, test, and compare different AI strategies.

## Games

### Implemented
- **Ultimate Tic-Tac-Toe** - A strategic variant of tic-tac-toe played on a 3x3 grid of tic-tac-toe boards
- **Connect Four** - The classic vertical grid game where players try to connect four tokens in a row

### In Development
- **Hearts** - Classic trick-taking card game
- **Gin Rummy** - Two-player card game variant of rummy

## Agent Framework

Each game supports multiple agent types:
- **Player Agent** - Interactive human player via terminal input
- **Random Agent** - Makes random valid moves (baseline for testing)
- **Custom Agents** - Extensible framework for implementing your own AI strategies

## Running Games

### Ultimate Tic-Tac-Toe
```bash
cargo run --bin ultimate_ttt
```

### Connect Four
```bash
cargo run --bin connect_four
```

## Project Structure

```
games-rs/
├── src/
│   ├── ultimate_ttt/     # Ultimate Tic-Tac-Toe implementation
│   │   └── agents/       # AI agents for Ultimate TTT
│   ├── connect_four/     # Connect Four implementation
│   │   └── agents/       # AI agents for Connect Four
│   ├── hearts/           # Hearts card game (in development)
│   ├── gin/              # Gin Rummy (in development)
│   ├── cards/            # Common card game utilities
│   ├── common/           # Shared game utilities
│   └── bin/              # Game executables
└── macros/               # Proc macros for game development
```

## Building Your Own Agent

Each game provides an `Agent` trait that you can implement to create your own AI:

```rust
pub trait Agent {
    fn get_move(&self, board: &Board) -> Move;
}
```

Implement this trait with your strategy (minimax, Monte Carlo Tree Search, neural networks, etc.) and plug it into any game.

## Development

### Prerequisites
- Rust 2024 edition or later

### Build
```bash
cargo build --release
```

### Test
```bash
cargo test
```

## License

This project is open source.
