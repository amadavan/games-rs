use crate::connect_four::{agents::Agent, board::{Board, Token}};

pub struct PlayerAgent {
  pub token: Token,
}

impl PlayerAgent {
    pub fn new(token: Token) -> Self {
        PlayerAgent { token }
    }
}

impl Agent for PlayerAgent {
    fn get_move(&self, board: &Board) -> usize {
        let mut mv = None;

        // Simple strategy: choose the first available column
        while mv.is_none() {
            println!("{:?}", board);
            println!("Enter which column (0-{}) to play:", 6);

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let col: usize = match input.trim().parse() {
                Ok(num) if num < 7 => num,
                _ => {
                    println!("Invalid input, try again.");
                    continue;
                }
            };

            if board.is_valid_move(col) {
                mv = Some(col);
            } else {
                println!("Column is full or invalid, try again.");
            }
        }

        mv.unwrap()
    }
}