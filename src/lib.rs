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
pub enum GameStatus {
    InProgress,
    Win(u8),
    Draw,
}

pub trait Game:
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
    const name: &'static str;

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
        + Into<u8>
        + Send
        + Sync;

    fn get_current_player(&self) -> Self::PlayerType;

    fn get_available_moves(&self) -> Vec<Self::MoveType>;

    fn play(&mut self, mv: Self::MoveType, player: Self::PlayerType) -> Result<(), String>;

    fn get_status(&self) -> GameStatus;

    fn move_message(&self) -> &str {
        ""
    }
}

/// A recorded game sample containing the sequence of moves and final result.
pub struct PlayThrough<G: Game> {
    result: GameStatus,
    moves: Vec<(<G as Game>::PlayerType, <G as Game>::MoveType)>,
}

impl<G: Game> PlayThrough<G> {
    pub fn new(
        result: GameStatus,
        moves: Vec<(<G as Game>::PlayerType, <G as Game>::MoveType)>,
    ) -> Self {
        PlayThrough { result, moves }
    }

    /// Returns the result of the game.
    pub fn get_result(&self) -> &GameStatus {
        &self.result
    }

    /// Returns the sequence of moves made during the game.
    pub fn get_moves(&self) -> &Vec<(<G as Game>::PlayerType, <G as Game>::MoveType)> {
        &self.moves
    }

    pub fn set_result(&mut self, result: GameStatus) -> () {
        self.result = result;
    }

    pub fn add_move(&mut self, player: <G as Game>::PlayerType, mv: <G as Game>::MoveType) -> () {
        self.moves.push((player, mv));
    }
}

impl<G: Game>
    From<(
        GameStatus,
        Vec<(<G as Game>::PlayerType, <G as Game>::MoveType)>,
    )> for PlayThrough<G>
where
    G: Game,
{
    fn from(
        value: (
            GameStatus,
            Vec<(<G as Game>::PlayerType, <G as Game>::MoveType)>,
        ),
    ) -> Self {
        PlayThrough {
            result: value.0,
            moves: value.1,
        }
    }
}

/// Plays a single game between two agents and returns the playthrough.
pub fn play_game<G: Game>(a1: &dyn agents::Agent<G>, a2: &dyn agents::Agent<G>) -> PlayThrough<G> {
    let mut game = G::default();
    let mut playthrough: PlayThrough<G> = PlayThrough::new(GameStatus::InProgress, Vec::new());

    loop {
        let current_player = game.get_current_player();

        let available_moves = game.get_available_moves();
        if available_moves.is_empty() {
            playthrough.set_result(GameStatus::Draw);
            return playthrough;
        }

        let move_to_play = if current_player == G::PlayerType::from(1) {
            a1.get_move(&game)
        } else {
            a2.get_move(&game)
        };

        let mv = move_to_play;
        game.play(mv, current_player).unwrap();
        playthrough.add_move(current_player, mv);

        if game.get_status() != GameStatus::InProgress {
            playthrough.set_result(game.get_status());
            return playthrough;
        }
    }
}
