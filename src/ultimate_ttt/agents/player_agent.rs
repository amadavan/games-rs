use std::io;
use crate::ultimate_ttt::board;
use super::Agent;

pub struct PlayerAgent {}

impl Agent for PlayerAgent {
    fn get_move(&self, board: &board::Board) -> board::Move {
        let mut mv = None;

        while mv.is_none() {
            println!("{:?}", board);
            println!("Enter your move as: <microboard_row> <microboard_col> <cell_row> <cell_col>");

            // Game logic would go here
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let coords: Vec<usize> = input
                .trim()
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();

            if coords.len() == 4 {
                let microboard_row = coords[0];
                let microboard_col = coords[1];
                let cell_row = coords[2];
                let cell_col = coords[3];

                // Validate and create move
                let proposed_move = (
                    microboard_row,
                    microboard_col,
                    cell_row,
                    cell_col,
                ).into();

                if board.is_valid_move(proposed_move) {
                    mv = Some(proposed_move);
                } else {
                    println!("Invalid move, try again.");
                }
            } else {
                println!("Please enter exactly four numbers.");
            }
        }

        mv.unwrap()
    }
}
