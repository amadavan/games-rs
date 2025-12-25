#![allow(warnings)]

use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

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
    Copy
    + Clone
    + std::hash::Hash
    + Eq
    + Default
    + Ord
    + Serialize
    + for<'de> Deserialize<'de>
    + Debug
    + Display
{
    type MoveType: Copy
        + Clone
        + std::hash::Hash
        + Eq
        + Ord
        + Serialize
        + for<'de> Deserialize<'de>
        + Debug
        + FromStr
        + Send
        + Sync;

    type PlayerType: Copy
        + Clone
        + std::hash::Hash
        + Eq
        + Ord
        + Serialize
        + for<'de> Deserialize<'de>
        + Debug
        + From<u8>
        + Into<u8>;

    fn get_current_player(&self) -> Self::PlayerType;

    fn get_available_moves(&self) -> Vec<Self::MoveType>;

    fn play(&mut self, mv: Self::MoveType, player: Self::PlayerType) -> Result<(), String>;

    fn get_status(&self) -> BoardStatus;
}

pub fn play_game<Game: GameBoard>(
    a1: &dyn agents::Agent<Game>,
    a2: &dyn agents::Agent<Game>,
) -> (BoardStatus, Vec<Game::MoveType>) {
    let mut game = Game::default();
    let mut mvs = Vec::new();

    loop {
        let current_player = game.get_current_player();

        let available_moves = game.get_available_moves();
        if available_moves.is_empty() {
            return (game.get_status(), mvs);
        }

        let move_to_play = if current_player == Game::PlayerType::from(1) {
            a1.get_move(&game)
        } else {
            a2.get_move(&game)
        };

        let mv = move_to_play;
        game.play(mv, current_player).unwrap();
        mvs.push(mv);

        if game.get_status() != BoardStatus::InProgress {
            return (game.get_status(), mvs);
        }
    }
}
