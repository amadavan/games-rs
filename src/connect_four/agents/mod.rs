use crate::connect_four::board::Board;

pub mod random_agent;
pub mod player_agent;

pub trait Agent {
    fn get_move(&self, board: &Board) -> usize;
}