use crate::ultimate_ttt::agents::Agent;
use rand::seq::IndexedRandom;
use crate::ultimate_ttt::Player;

pub struct RandomAgent {
    pub player: Player,
}

impl RandomAgent {
    pub fn new(player: Player) -> Self {
        RandomAgent { player }
    }
}

impl Agent for RandomAgent {
    fn get_move(
        &self,
        board: &crate::ultimate_ttt::board::Board,
    ) -> crate::ultimate_ttt::board::Move {
        let available_moves = board.get_available_moves();
        let mut rng = rand::rng();
        available_moves.choose(&mut rng).unwrap().clone()
    }
}
