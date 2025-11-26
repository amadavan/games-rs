use crate::connect_four::board::{Board, BoardStatus, Token};

pub mod mcgs_agent;
pub mod player_agent;
pub mod random_agent;

pub trait Agent {
    fn get_move(&mut self, board: &Board, prev_moves: &Vec<(Token, Board)>) -> usize;

    fn notify_win(&mut self, moves: &Vec<(Token, Board)>, status: BoardStatus) -> ();
}
