use crate::connect_four::board::{Board, Token};
use rand::Rng;
use super::Agent;

pub struct RandomAgent {
  pub token: Token,
}

impl RandomAgent {
    pub fn new(token: Token) -> Self {
        RandomAgent { token }
    }
}

impl Agent for RandomAgent {
    fn get_move(&self, board: &Board) -> usize {
        let mut rng = rand::rng();
        let mut col = rng.random_range(0..7);

        while !board.is_valid_move(col) {
            col = rng.random_range(0..7);
        }

        col
    }
}