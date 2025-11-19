use crate::ultimate_ttt::board;
pub mod player_agent;
pub mod random_agent;

pub trait Agent {
    fn get_move(&self, board: &board::Board) -> board::Move;
}