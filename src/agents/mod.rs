//! AI agents for playing board games.
//!
//! This module provides various agent implementations that can play games
//! implementing the `GameBoard` trait. Agents range from human players to
//! sophisticated Monte Carlo graph search algorithms.

pub mod monte_carlo_graph;
pub mod scorer;
pub mod train;

use rand::Rng;
use rand::seq::IndexedRandom;
use std::cmp::max;
use std::cmp::min;

use crate::{Game, GameStatus, agents::monte_carlo_graph::MonteCarloGraph};

/// Trait for game-playing agents.
///
/// Agents can select moves and optionally receive feedback about game outcomes
/// to improve their strategy.
pub trait Agent<G: Game> {
    /// Selects a move for the current board state.
    ///
    /// # Arguments
    /// * `board` - The current game state
    ///
    /// # Returns
    /// The selected move for the current board state.
    fn get_move(&self, board: &G) -> <G as Game>::MoveType;
}

/// An interactive agent that prompts a human player for moves via stdin.
///
/// This agent displays the current board state and requests move input from
/// the user through the console.
pub struct PlayerAgent<G: Game> {
    /// The player number this agent represents
    pub player: u8,
    _marker: std::marker::PhantomData<G>,
}

impl<G: Game> PlayerAgent<G> {
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

impl<G: Game> Agent<G> for PlayerAgent<G> {
    /// Prompts the human player to enter a move via stdin.
    ///
    /// Displays the board and repeatedly requests input until a valid move is entered.
    fn get_move(&self, board: &G) -> <G as Game>::MoveType {
        let mut mv = None;

        while mv.is_none() {
            println!("{}", board);
            println!("Player {}, enter your move:", self.player);

            // Get the user input
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let parsed_move: Result<<G as Game>::MoveType, _> = input.trim().parse();

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
pub struct RandomAgent<G: Game> {
    _marker: std::marker::PhantomData<G>,
}

impl<G: Game> RandomAgent<G> {
    /// Creates a new random agent.
    pub fn new() -> Self {
        RandomAgent {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<G: Game> Agent<G> for RandomAgent<G> {
    /// Selects a random move from the available moves.
    fn get_move(&self, board: &G) -> <G as Game>::MoveType {
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
pub struct MonteCarloGraphSearch<G: Game> {
    graph: MonteCarloGraph<G>,
}

impl<G: Game> MonteCarloGraphSearch<G> {
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
    pub fn from_graph(graph: MonteCarloGraph<G>) -> Self {
        MonteCarloGraphSearch { graph }
    }
}

impl<G: Game> Agent<G> for MonteCarloGraphSearch<G> {
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
    fn get_move(&self, board: &G) -> <G as Game>::MoveType {
        let available_moves = board.get_available_moves();

        let values = available_moves
            .iter()
            .map(|mv| {
                let next_board = {
                    let mut next_board = board.clone();
                    let _ = next_board.play(*mv, board.get_current_player());
                    next_board
                };
                let edge_weight = self.graph.edge_weight(board.clone(), next_board.clone());
                if edge_weight.is_none() {
                    (mv, 1f64 + 2f64.sqrt())
                } else {
                    let edge_weight = edge_weight.unwrap();
                    let target_count = self.graph.get_aggregate_outcomes(&next_board).simulations();
                    let w = (edge_weight.wins() + 1) as f64;
                    let n = (edge_weight.simulations() + 1) as f64;
                    let N = (target_count + 1) as f64;
                    (mv, w / n + (2.0 * N.ln() / n).sqrt())
                }
            })
            .collect::<Vec<_>>();

        let max = values
            .iter()
            .max_by(|a, b| b.1.partial_cmp(&a.1).unwrap())
            .unwrap()
            .1;

        let maximizers = values
            .iter()
            .filter(|(_, v)| *v > max - 1e-6)
            .collect::<Vec<_>>();

        let mut random = rand::rng();
        let choice = maximizers[random.random_range(0..maximizers.len())];

        *choice.0
    }
}

pub trait ScoreFunction<G: Game> {
    fn score(&self, board: &G, mv: &<G as Game>::MoveType, player: G::PlayerType) -> f32;

    fn update(&mut self, _moves: &Vec<(u8, G)>, _status: GameStatus) -> () {
        ()
    }
}

pub struct MinimaxAgent<G: Game, ScoreFn: ScoreFunction<G>> {
    depth: usize,
    score_fn: ScoreFn,
    _marker: std::marker::PhantomData<G>,
}

impl<G: Game, ScoreFn: ScoreFunction<G>> MinimaxAgent<G, ScoreFn> {
    pub fn new(depth: usize, score_fn: ScoreFn) -> Self {
        MinimaxAgent {
            depth,
            score_fn,
            _marker: std::marker::PhantomData,
        }
    }

    fn alpha_beta(
        &self,
        board: &G,
        mv: G::MoveType,
        depth: usize,
        alpha: f32,
        beta: f32,
        player: G::PlayerType,
    ) -> f32 {
        if depth == 0 || board.get_status() != GameStatus::InProgress {
            let sign = if player == board.get_current_player() {
                1.0
            } else {
                -1.0
            };
            return sign * self.score_fn.score(board, &mv, board.get_current_player());
        }

        let mut alpha = alpha;
        let mut beta = beta;

        if player == board.get_current_player() {
            let mut max_eval = f32::NEG_INFINITY;
            for mv in board.get_available_moves() {
                let mut new_board = board.clone();
                new_board.play(mv, board.get_current_player()).unwrap();
                let eval = self.alpha_beta(&new_board, mv, depth - 1, alpha, beta, player);
                max_eval = f32::max(max_eval, eval);
                alpha = f32::max(alpha, eval);
                if beta <= alpha {
                    break;
                }
            }
            max_eval
        } else {
            let mut min_eval = f32::INFINITY;
            for mv in board.get_available_moves() {
                let mut new_board = board.clone();
                new_board.play(mv, board.get_current_player()).unwrap();
                let eval = self.alpha_beta(&new_board, mv, depth - 1, alpha, beta, player);
                min_eval = f32::min(min_eval, eval);
                beta = f32::min(beta, eval);
                if beta <= alpha {
                    break;
                }
            }
            min_eval
        }
    }
}

impl<G: Game, ScoreFn: ScoreFunction<G>> Agent<G> for MinimaxAgent<G, ScoreFn> {
    fn get_move(&self, board: &G) -> <G as Game>::MoveType {
        let available_moves = board.get_available_moves();

        let mut best_move = available_moves[0];
        let mut best_score = f32::NEG_INFINITY;

        for mv in available_moves {
            let score = self.alpha_beta(
                &{
                    let mut tmp_board = board.clone();
                    tmp_board.play(mv, board.get_current_player()).unwrap();
                    tmp_board
                },
                mv,
                self.depth - 1,
                f32::NEG_INFINITY,
                f32::INFINITY,
                board.get_current_player(),
            );

            if score > best_score {
                best_score = score;
                best_move = mv;
            }
        }

        best_move
    }
}
