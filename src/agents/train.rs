//! Training utilities for game-playing agents.

use indicatif::{MultiProgress, ProgressStyle};
use rayon::prelude::*;
use std::sync::Arc;

use crate::{
    Game, GameStatus, PlayThrough,
    agents::{self, monte_carlo_graph::MonteCarloGraph},
    common::defaults,
    play_game,
};

pub trait TrainableComponent<G: Game> {
    const name: &'static str;

    fn train(&mut self, samples: &PlayThrough<G>, verbose: bool) -> ();

    fn train_batch(
        &mut self,
        samples_batch: &Vec<PlayThrough<G>>,
        mpb: Option<&MultiProgress>,
    ) -> () {
        let pb = if let Some(mpb) = mpb {
            let pb = mpb
                .add(indicatif::ProgressBar::new(samples_batch.len() as u64))
                .with_style(defaults::PB_STYLE.clone())
                .with_prefix(format!("{}/{}", G::name, Self::name));
            pb.enable_steady_tick(std::time::Duration::from_millis(100));
            Some(pb)
        } else {
            None
        };

        for sample in samples_batch {
            if let Some(pb) = &pb {
                pb.inc(1);
            }
            self.train(sample, mpb.is_some());
        }

        if let Some(pb) = &pb {
            pb.finish();
        }
    }
}

/// Plays multiple games sequentially between two agents and collects samples.
pub fn play_batch<G: Game>(
    agent1: &dyn agents::Agent<G>,
    agent2: &dyn agents::Agent<G>,
    num_games: usize,
    mpb: Option<&MultiProgress>,
) -> Vec<PlayThrough<G>> {
    let pb = if let Some(mpb) = mpb {
        let pb = mpb
            .add(indicatif::ProgressBar::new(num_games as u64))
            .with_style(defaults::PB_STYLE.clone())
            .with_prefix("Batch plays");
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

                play_game::<G>(agent1, agent2).into()
            })
            .collect::<Vec<PlayThrough<G>>>()
    };

    if let Some(pb) = &pb {
        pb.finish();
    }

    results
}

/// Plays multiple games in parallel using agent factories and collects samples.
/// Factories are needed to create thread-local agent instances.
pub fn play_batch_parallel<G: Game, F1, F2>(
    agent1_factory: F1,
    agent2_factory: F2,
    num_games: usize,
    mpb: Option<&MultiProgress>,
) -> Vec<PlayThrough<G>>
where
    G: Send,
    G::MoveType: Send,
    F1: Fn() -> Box<dyn agents::Agent<G>> + Sync,
    F2: Fn() -> Box<dyn agents::Agent<G>> + Sync,
{
    let pb = if let Some(mpb) = mpb {
        let pb = mpb
            .add(indicatif::ProgressBar::new(num_games as u64))
            .with_style(defaults::PB_STYLE.clone())
            .with_prefix("Batch plays");
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

            play_game::<G>(agent1_factory().as_ref(), agent2_factory().as_ref()).into()
        })
        .collect::<Vec<PlayThrough<G>>>();

    if let Some(pb) = &pb {
        pb.finish();
    }

    results
}
