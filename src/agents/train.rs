//! Training utilities for game-playing agents.

use rayon::prelude::*;
use std::sync::Arc;

use crate::{
    BoardStatus, GameBoard,
    agents::{self, monte_carlo_graph::MonteCarloGraph},
    play_game,
};

/// A recorded game sample containing the sequence of moves and final result.
pub struct Sample<Game: GameBoard> {
    result: BoardStatus,
    moves: Vec<<Game as GameBoard>::MoveType>,
}

impl<Game: GameBoard> From<(BoardStatus, Vec<<Game as GameBoard>::MoveType>)> for Sample<Game>
where
    Game: GameBoard,
{
    fn from(value: (BoardStatus, Vec<<Game as GameBoard>::MoveType>)) -> Self {
        Sample {
            moves: value.1,
            result: value.0,
        }
    }
}

/// Plays multiple games sequentially between two agents and collects samples.
pub fn play_batch<Game: GameBoard>(
    agent1: &dyn agents::Agent<Game>,
    agent2: &dyn agents::Agent<Game>,
    num_games: usize,
    verbose: bool,
) -> Vec<Sample<Game>> {
    let pb = if verbose {
        let pb = indicatif::ProgressBar::new(num_games as u64);
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        Some(pb)
    } else {
        None
    };

    let results = {
        (0..num_games)
            .into_iter()
            .map(|_| {
                if let Some(pb) = &pb {
                    pb.inc(1);
                }

                play_game::<Game>(agent1, agent2).into()
            })
            .collect::<Vec<Sample<Game>>>()
    };

    if let Some(pb) = &pb {
        pb.finish();
    }

    results
}

/// Plays multiple games in parallel using agent factories and collects samples.
/// Factories are needed to create thread-local agent instances.
pub fn play_batch_parallel<Game: GameBoard, F1, F2>(
    agent1_factory: F1,
    agent2_factory: F2,
    num_games: usize,
    verbose: bool,
) -> Vec<Sample<Game>>
where
    Game: Send,
    Game::MoveType: Send,
    F1: Fn() -> Box<dyn agents::Agent<Game>> + Sync,
    F2: Fn() -> Box<dyn agents::Agent<Game>> + Sync,
{
    let pb = if verbose {
        let pb = indicatif::ProgressBar::new(num_games as u64);
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        Some(Arc::new(pb))
    } else {
        None
    };

    let results = (0..num_games)
        .into_par_iter()
        .map(|_| {
            if let Some(pb) = &pb {
                pb.inc(1);
            }

            play_game::<Game>(agent1_factory().as_ref(), agent2_factory().as_ref()).into()
        })
        .collect::<Vec<Sample<Game>>>();

    if let Some(pb) = &pb {
        pb.finish();
    }

    results
}

/// Trains a Monte Carlo Graph Search agent by backpropagating results from game samples.
pub fn train_MCGS<Game: GameBoard>(
    mcg: &mut MonteCarloGraph<Game>,
    samples: &Vec<Sample<Game>>,
    verbose: bool,
) -> () {
    let pb = if verbose {
        Some(indicatif::ProgressBar::new(samples.len() as u64))
    } else {
        None
    };

    for sample in samples {
        if let Some(pb) = &pb {
            pb.inc(1);
        }

        // Generate board states from moves
        let board_states = {
            let mut game = Game::default();
            let mut states = Vec::new();
            states.push(game.clone());

            for mv in &sample.moves {
                let current_player = game.get_current_player();
                game.play(*mv, current_player).unwrap();
                states.push(game.clone());
            }

            states
        };

        // Backpropogate result
        mcg.back_propogate(board_states, sample.result);
    }

    if let Some(pb) = &pb {
        pb.finish();
    }
}
