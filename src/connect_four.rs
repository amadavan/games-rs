use core::fmt;

use serde::{Deserialize, Serialize};

use crate::{BoardStatus, GameBoard};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub enum Token {
    Empty,
    Red,
    Yellow,
}

impl Token {
    pub fn as_u8(&self) -> u8 {
        match self {
            Token::Empty => 0,
            Token::Red => 1,
            Token::Yellow => 2,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct ConnectFour {
    grid: [[Token; 7]; 6],
}

impl ConnectFour {
    pub fn new() -> Self {
        ConnectFour {
            grid: [[Token::Empty; 7]; 6],
        }
    }

    pub fn is_valid_move(&self, column: usize) -> bool {
        if column >= 7 {
            return false;
        }
        self.grid[5][column] == Token::Empty
    }
}

impl GameBoard for ConnectFour {
    type MoveType = usize;

    fn get_current_player(&self) -> u8 {
        let mut x_count = 0;
        let mut o_count = 0;

        for row in 0..6 {
            for col in 0..7 {
                match self.grid[row][col] {
                    Token::Red => x_count += 1,
                    Token::Yellow => o_count += 1,
                    Token::Empty => {}
                }
            }
        }

        if x_count <= o_count {
            1 // Player Red's turn
        } else {
            2 // Player Yellow's turn
        }
    }

    fn get_available_moves(&self) -> Vec<Self::MoveType> {
        let mut moves = Vec::new();
        for col in 0..7 {
            if self.grid[5][col] == Token::Empty {
                moves.push(col);
            }
        }
        moves
    }

    fn play(&mut self, mv: Self::MoveType, player: impl Into<u8>) -> Result<(), String> {
        if mv >= 7 || self.grid[5][mv] != Token::Empty {
            return Err("Invalid move".to_string());
        }

        let player_token = match player.into() {
            1 => Token::Red,
            _ => Token::Yellow,
        };

        for row in 0..6 {
            if self.grid[row][mv] == Token::Empty {
                self.grid[row][mv] = player_token;
                break;
            }
        }

        Ok(())
    }

    fn get_status(&self) -> BoardStatus {
        // Implementation to determine the current status of the board
        let mut status = BoardStatus::InProgress; // Placeholder

        // Check columns for 4 in a row
        for col in 0..7 {
            for row in 0..3 {
                if self.grid[row][col] != Token::Empty
                    && self.grid[row][col] == self.grid[row + 1][col]
                    && self.grid[row][col] == self.grid[row + 2][col]
                    && self.grid[row][col] == self.grid[row + 3][col]
                {
                    return BoardStatus::Win(self.grid[row][col].as_u8());
                }
            }
        }

        // Check rows for 4 in a row
        for row in 0..6 {
            for col in 0..4 {
                if self.grid[row][col] != Token::Empty
                    && self.grid[row][col] == self.grid[row][col + 1]
                    && self.grid[row][col] == self.grid[row][col + 2]
                    && self.grid[row][col] == self.grid[row][col + 3]
                {
                    return BoardStatus::Win(self.grid[row][col].as_u8());
                }
            }
        }

        // Check diagonals for 4 in a row
        for row in 0..3 {
            for col in 0..4 {
                if self.grid[row][col] != Token::Empty
                    && self.grid[row][col] == self.grid[row + 1][col + 1]
                    && self.grid[row][col] == self.grid[row + 2][col + 2]
                    && self.grid[row][col] == self.grid[row + 3][col + 3]
                {
                    return BoardStatus::Win(self.grid[row][col].as_u8());
                }
            }
        }
        for row in 0..3 {
            for col in 3..7 {
                if self.grid[row][col] != Token::Empty
                    && self.grid[row][col] == self.grid[row + 1][col - 1]
                    && self.grid[row][col] == self.grid[row + 2][col - 2]
                    && self.grid[row][col] == self.grid[row + 3][col - 3]
                {
                    return BoardStatus::Win(self.grid[row][col].as_u8());
                }
            }
        }

        // Check for draw
        if self
            .grid
            .iter()
            .all(|row| row.iter().all(|&cell| cell != Token::Empty))
        {
            status = BoardStatus::Draw;
        }

        status
    }
}

impl Default for ConnectFour {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for ConnectFour {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in (0..6).rev() {
            for col in 0..7 {
                let symbol = match self.grid[row][col] {
                    Token::Empty => '.',
                    Token::Red => 'R',
                    Token::Yellow => 'Y',
                };
                write!(f, "{} ", symbol)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
