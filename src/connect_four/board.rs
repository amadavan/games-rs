use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    Empty,
    Red,
    Yellow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        let mut status = BoardStatus::InProgress; // Placeholder

        // Check columns for 4 in a row
        for col in 0..7 {
            for row in 0..3 {
                if self.grid[row][col] != Token::Empty
                    && self.grid[row][col] == self.grid[row + 1][col]
                    && self.grid[row][col] == self.grid[row + 2][col]
                    && self.grid[row][col] == self.grid[row + 3][col]
                {
                    return BoardStatus::Win(self.grid[row][col]);
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
                    return BoardStatus::Win(self.grid[row][col]);
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
                    return BoardStatus::Win(self.grid[row][col]);
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
                    return BoardStatus::Win(self.grid[row][col]);
                }
            }
        }

        // Check for draw
        if self.grid.iter().all(|row| row.iter().all(|&cell| cell != Token::Empty)) {
            status = BoardStatus::Draw;
        }

        
        status
    }

    pub fn is_valid_move(&self, column: usize) -> bool {
        if column >= 7 {
            return false;
        }
        self.grid[5][column] == Token::Empty
    }

    pub fn get_available_moves(&self) -> [bool; 7] {
        let mut moves = [false; 7];
        for col in 0..7 {
            if self.grid[5][col] == Token::Empty {
                moves[col] = true;
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

impl fmt::Debug for Board {
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