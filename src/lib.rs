use std::{fmt::Debug, str::FromStr};

use serde::{Deserialize, Serialize};

pub mod agents;
pub mod cards;
pub mod common;
pub mod connect_four;
pub mod rummy;
pub mod ultimate_ttt;

extern crate macros;

mod derive_alias {
    derive_aliases::define! {
        Eq = ::core::cmp::Eq, ::core::cmp::PartialEq;
        Ord = ::core::cmp::PartialOrd, ::core::cmp::Ord, ..Eq;
        Copy = ::core::marker::Copy, ::core::clone::Clone;
        StdTraits = ..Eq, ..Ord, ..Copy, ::core::hash::Hash;
        // Serialize = ::serde::Serialize, ::serde::Deserialize, ..StdTraits;
    }
}

use derive_aliases::derive;

#[derive(..StdTraits)]
pub enum BoardStatus {
    InProgress,
    Win(u8),
    Draw,
}

pub trait GameBoard:
    Copy + Clone + std::hash::Hash + Eq + Default + Ord + Serialize + for<'de> Deserialize<'de> + Debug
{
    type MoveType: Copy
        + Clone
        + std::hash::Hash
        + Eq
        + Ord
        + Serialize
        + for<'de> Deserialize<'de>
        + Debug
        + FromStr;

    fn get_current_player(&self) -> u8;

    fn get_available_moves(&self) -> Vec<Self::MoveType>;

    fn play(&mut self, mv: Self::MoveType, player: impl Into<u8>) -> Result<(), String>;

    fn get_status(&self) -> BoardStatus;
}

pub fn play_game<Game: GameBoard, A1: agents::Agent<Game>, A2: agents::Agent<Game>>(
    a1: &A1,
    a2: &A2,
) -> BoardStatus {
    let mut game = Game::default();
    let mut current_player = game.get_current_player();

    loop {
        let available_moves = game.get_available_moves();
        if available_moves.is_empty() {
            return game.get_status();
        }

        let move_to_play = if current_player == 1 {
            a1.get_move(&game)
        } else {
            a2.get_move(&game)
        };

        let mv = move_to_play;
        game.play(mv, current_player).unwrap();
        current_player = game.get_current_player();

        if game.get_status() != BoardStatus::InProgress {
            return game.get_status();
        }
    }
}
