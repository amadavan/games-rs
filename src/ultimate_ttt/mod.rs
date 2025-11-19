use serde::{Deserialize, Serialize};

use crate::ultimate_ttt::board::BoardStatus;

pub mod agents;
pub mod board;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Player {
    X,
    O,
    Empty,
}

impl bincode::Encode for Player {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        match self {
            Player::X => 1u8.encode(encoder),
            Player::O => 2u8.encode(encoder),
            Player::Empty => 0u8.encode(encoder),
        }
    }
}

impl<DC> bincode::Decode<DC> for Player {
    fn decode<D: bincode::de::Decoder>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let value = u8::decode(decoder)?;
        match value {
            1 => Ok(Player::X),
            2 => Ok(Player::O),
            0 => Ok(Player::Empty),
            _ => Err(bincode::error::DecodeError::OtherString(format!(
                "Invalid value for Player: {}",
                value
            ))),
        }
    }
}

impl From<Player> for char {
    fn from(state: Player) -> Self {
        match state {
            Player::X => 'X',
            Player::O => 'O',
            Player::Empty => '-',
        }
    }
}

pub fn play_game<A1: agents::Agent, A2: agents::Agent>(a1: &A1, a2: &A2) -> BoardStatus {
    let mut board = board::Board::new();

    let mut current_player = Player::X;

    while board.get_status() == &board::BoardStatus::InProgress {
        let mv = match current_player {
            Player::X => a1.get_move(&board),
            Player::O => a2.get_move(&board),
            Player::Empty => panic!("Empty player cannot make a move"),
        };

        let play = board.play(mv, current_player);
        if play.is_err() {
            println!("Invalid move attempted by {:?}: {:?}", current_player, mv);
            continue;
        }

        current_player = match current_player {
            Player::X => Player::O,
            Player::O => Player::X,
            Player::Empty => panic!("Empty player cannot make a move"),
        };
    }

    board.get_status().clone()
}
