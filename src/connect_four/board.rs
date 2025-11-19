use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    Empty,
    Red,
    Yellow,
}

pub enum BoardStatus {
    InProgress,
    Draw,
    Win(Token),
}

pub struct Board {
    grid: [[Token; 7]; 6],
}

impl Board {
    pub fn new() -> Self {
        Board {
            grid: [[Token::Empty; 7]; 6],
        }
    }

    pub fn get_status(&self) -> BoardStatus {
        // Implementation to determine the current status of the board
        BoardStatus::InProgress // Placeholder

        // TODO: check for 4 in a row, draw, etc.
    }

    pub fn get_available_moves(&self) -> [bool; 7] {
        let mut moves = [false; 7];
        for col in 0..7 {
            for row in 0..6 {
                if self.grid[row][col] == Token::Empty {
                    moves[col] = true;
                }
            }
        }
        moves
    }

    pub fn play(&mut self, column: usize, token: Token) -> Result<(), String> {
        if column >= 7 {
            return Err("Invalid column".to_string());
        }

        for row in 0..6 {
            if self.grid[row][column] == Token::Empty {
                self.grid[row][column] = token;
                return Ok(());
            }
        }

        Err("Column is full".to_string())
    }
}

impl fmt::Display for Board {
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