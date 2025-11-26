use crate::connect_four::board::{Board, Token};

pub mod player_agent;
pub mod random_agent;

pub trait Agent {
    fn get_move(&self, board: &Board, prev_moves: &Vec<(Token, Board)>) -> usize;
}
