//! AI agents for playing board games.
//!
//! This module provides various agent implementations that can play games
//! implementing the `GameBoard` trait. Agents range from human players to
//! sophisticated Monte Carlo graph search algorithms.

pub mod monte_carlo_graph;
pub mod scorer;

use rand::seq::IndexedRandom;
use std::cmp::max;
use std::cmp::min;

use crate::{BoardStatus, GameBoard, agents::monte_carlo_graph::MonteCarloGraph};

/// Trait for game-playing agents.
///
/// Agents can select moves and optionally receive feedback about game outcomes
/// to improve their strategy.
pub trait Agent<Game: GameBoard> {
    /// Selects a move for the current board state.
    ///
    /// # Arguments
    /// * `board` - The current game state
    ///
    /// # Returns
    /// The selected move for the current board state.
    fn get_move(&self, board: &Game) -> <Game as GameBoard>::MoveType;

    /// Notifies the agent about game progression and outcome.
    ///
    /// This method is called to provide feedback that learning agents can use
    /// to update their strategies.
    ///
    /// # Arguments
    /// * `_moves` - The sequence of (player, board state) pairs representing the game history
    /// * `_status` - The final game status (win, draw, or in progress)
    fn notify(&mut self, _moves: &Vec<(u8, Game)>, _status: BoardStatus) -> () {
        ()
    }
}

/// An interactive agent that prompts a human player for moves via stdin.
///
/// This agent displays the current board state and requests move input from
/// the user through the console.
pub struct PlayerAgent<Game: GameBoard> {
    /// The player number this agent represents
    pub player: u8,
    _marker: std::marker::PhantomData<Game>,
}

impl<Game: GameBoard> PlayerAgent<Game> {
    /// Creates a new human player agent.
    ///
    /// # Arguments
    /// * `player` - The player number (1 or 2)
    pub fn new(player: u8) -> Self {
        PlayerAgent {
            player,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<Game: GameBoard> Agent<Game> for PlayerAgent<Game> {
    /// Prompts the human player to enter a move via stdin.
    ///
    /// Displays the board and repeatedly requests input until a valid move is entered.
    fn get_move(&self, board: &Game) -> <Game as GameBoard>::MoveType {
        let mut mv = None;

        while mv.is_none() {
            println!("{:?}", board);
            println!("Player {}, enter your move:", self.player);

            // Get the user input
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let parsed_move: Result<<Game as GameBoard>::MoveType, _> = input.trim().parse();

            if let Ok(parsed_move) = parsed_move
                && board.get_available_moves().contains(&parsed_move)
            {
                mv = Some(parsed_move);
            } else {
                println!("Invalid move, try again.");
            }
        }

        mv.unwrap()
    }
}

/// An agent that selects moves uniformly at random from available moves.
///
/// This agent provides a baseline for comparison with more sophisticated strategies.
pub struct RandomAgent<Game: GameBoard> {
    _marker: std::marker::PhantomData<Game>,
}

impl<Game: GameBoard> RandomAgent<Game> {
    /// Creates a new random agent.
    pub fn new() -> Self {
        RandomAgent {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<Game: GameBoard> Agent<Game> for RandomAgent<Game> {
    /// Selects a random move from the available moves.
    fn get_move(&self, board: &Game) -> <Game as GameBoard>::MoveType {
        let available_moves = board.get_available_moves();
        let mut rng = rand::rng();
        available_moves.choose(&mut rng).unwrap().clone()
    }
}

/// An agent using Monte Carlo Graph Search with UCT (Upper Confidence bounds applied to Trees).
///
/// This agent maintains a graph of game states and transitions, learning from game outcomes
/// to make increasingly better decisions. It uses the UCT formula to balance exploration
/// and exploitation when selecting moves.
pub struct MonteCarloGraphSearch<Game: GameBoard> {
    graph: MonteCarloGraph<Game>,
}

impl<Game: GameBoard> MonteCarloGraphSearch<Game> {
    /// Creates a new Monte Carlo Graph Search agent with an empty graph.
    pub fn new() -> Self {
        MonteCarloGraphSearch {
            graph: MonteCarloGraph::new(),
        }
    }

    /// Creates a Monte Carlo Graph Search agent from an existing graph.
    ///
    /// This allows loading a pre-trained graph to continue learning or use learned strategies.
    ///
    /// # Arguments
    /// * `graph` - A pre-existing Monte Carlo graph
    pub fn from_graph(graph: MonteCarloGraph<Game>) -> Self {
        MonteCarloGraphSearch { graph }
    }
}

impl<Game: GameBoard> Agent<Game> for MonteCarloGraphSearch<Game> {
    /// Selects a move using the UCT (Upper Confidence bounds applied to Trees) formula.
    ///
    /// For each available move, calculates a UCT value that balances:
    /// - Exploitation: moves with high win rates
    /// - Exploration: moves that haven't been tried much
    ///
    /// The formula used is: w/n + sqrt(2 * ln(N) / n)
    /// where w = wins, n = simulations for this move, N = total simulations from resulting state.
    ///
    /// Returns a random choice among the highest-valued moves.
    fn get_move(&self, board: &Game) -> <Game as GameBoard>::MoveType {
        let available_moves = board.get_available_moves();

        let values = available_moves
            .iter()
            .map(|mv| {
                let mut next_board = board.clone();
                let _ = next_board.play(*mv, board.get_current_player());
                let edge_weight = self.graph.edge_weight(board.clone(), next_board.clone());
                if edge_weight.is_none() {
                    (mv, 1f64 + 2f64.sqrt())
                } else {
                    let edge_weight = edge_weight.unwrap();
                    let target_count = self
                        .graph
                        .edges_from(&next_board)
                        .iter()
                        .map(|(_, (_, n))| *n)
                        .sum::<usize>();
                    let w = (edge_weight.0 + 1) as f64;
                    let n = (edge_weight.1 + 1) as f64;
                    let N = (target_count + 1) as f64;
                    (mv, w / n + (2.0 * N.ln() / n).sqrt())
                }
            })
            .collect::<Vec<_>>();

        let mut ordered_values = values.into_iter().collect::<Vec<_>>();
        ordered_values.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let maximizers = ordered_values
            .iter()
            .filter(|(_, v)| *v < ordered_values[0].1 + 1e-6)
            .collect::<Vec<_>>();
        use rand::Rng;
        let mut random = rand::rng();
        let choice = maximizers[random.random_range(0..maximizers.len())];

        choice.0.clone()
    }

    /// Updates the graph with game outcome information.
    ///
    /// Propagates win/loss statistics backward through the game state graph,
    /// allowing the agent to learn from the game's result.
    ///
    /// # Arguments
    /// * `_moves` - The sequence of (player, board state) pairs from the game
    /// * `_status` - The final game status (win or draw)
    fn notify(&mut self, _moves: &Vec<(u8, Game)>, _status: BoardStatus) -> () {
        let path = _moves.iter().map(|(_, b)| b.clone()).collect();
        self.graph.back_propogate(path, _status);
    }
}

pub trait ScoreFunction<Game: GameBoard> {
    fn score(&self, board: &Game, mv: &<Game as GameBoard>::MoveType, player: u8) -> f32;

    fn update(&mut self, _moves: &Vec<(u8, Game)>, _status: BoardStatus) -> () {
        ()
    }
}

pub struct MinimaxAgent<Game: GameBoard, ScoreFn: ScoreFunction<Game>> {
    depth: usize,
    score_fn: ScoreFn,
    _marker: std::marker::PhantomData<Game>,
}

impl<Game: GameBoard, ScoreFn: ScoreFunction<Game>> MinimaxAgent<Game, ScoreFn> {
    pub fn new(depth: usize, score_fn: ScoreFn) -> Self {
        MinimaxAgent {
            depth,
            score_fn,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<Game: GameBoard, ScoreFn: ScoreFunction<Game>> Agent<Game> for MinimaxAgent<Game, ScoreFn> {
    fn get_move(&self, board: &Game) -> <Game as GameBoard>::MoveType {
        let available_moves = board.get_available_moves();

        let mut best_move = available_moves[0];
        let mut best_score = f32::NEG_INFINITY;

        // TODO: need to traverse deeper (otherwise it only considers itself)
        for mv in available_moves {
            let mut score = self.score_fn.score(board, &mv, board.get_current_player());
            let mut tmp_board = board.clone();
            tmp_board.play(mv, board.get_current_player()).unwrap();

            for _ in 0..self.depth {
                if tmp_board.get_status() != BoardStatus::InProgress {
                    break;
                }

                let available_moves = tmp_board.get_available_moves();
                let scores = available_moves
                    .iter()
                    .map(|x| {
                        (
                            x,
                            self.score_fn
                                .score(&tmp_board, &x, tmp_board.get_current_player()),
                        )
                    })
                    .collect::<Vec<_>>();

                // Select the move with the highest score
                let mv_ = scores
                    .iter()
                    .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                    .unwrap();

                let sign = if board.get_current_player() == tmp_board.get_current_player() {
                    1.0
                } else {
                    -1.0
                };

                score = if board.get_current_player() == tmp_board.get_current_player() {
                    if score > sign * mv_.1 {
                        sign * mv_.1
                    } else {
                        score
                    }
                } else {
                    if score < sign * mv_.1 {
                        score
                    } else {
                        sign * mv_.1
                    }
                };

                tmp_board
                    .play(mv_.0.clone(), tmp_board.get_current_player())
                    .unwrap();
            }

            if score > best_score {
                best_score = score;
                best_move = mv;
            }
        }

        best_move
    }

    fn notify(&mut self, _moves: &Vec<(u8, Game)>, _status: BoardStatus) -> () {
        self.score_fn.update(_moves, _status);
    }
}
