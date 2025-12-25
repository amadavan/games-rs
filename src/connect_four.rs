//! Connect Four game implementation.
//!
//! This module implements the classic Connect Four game, where players take turns
//! dropping tokens into a vertical grid. The first player to connect four of their
//! tokens in a row (horizontally, vertically, or diagonally) wins the game.

use core::fmt;

use serde::{Deserialize, Serialize};

use crate::{BoardStatus, GameBoard};
use derive_aliases::derive;

/// Represents a token in the Connect Four game.
///
/// Tokens can be empty, red (player 1), or yellow (player 2).
#[derive(..StdTraits, Debug, Serialize, Deserialize)]
pub enum Token {
    /// An empty cell
    Empty,
    /// Red token (player 1)
    Red,
    /// Yellow token (player 2)
    Yellow,
}

impl From<u8> for Token {
    fn from(value: u8) -> Self {
        match value {
            1 => Token::Red,
            2 => Token::Yellow,
            _ => Token::Empty,
        }
    }
}

impl Into<u8> for Token {
    fn into(self) -> u8 {
        match self {
            Token::Red => 1,
            Token::Yellow => 2,
            Token::Empty => 0,
        }
    }
}

/// The Connect Four game board.
///
/// A 6-row by 7-column grid where tokens drop to the lowest available position
/// in each column. The board is indexed with row 0 at the bottom.
#[derive(..StdTraits, Serialize, Deserialize)]
pub struct ConnectFour {
    grid: [[Token; 7]; 6],
}

impl ConnectFour {
    /// Creates a new Connect Four game with an empty board.
    pub fn new() -> Self {
        ConnectFour {
            grid: [[Token::Empty; 7]; 6],
        }
    }

    /// Returns a reference to the current grid state.
    ///
    /// # Returns
    /// A 2D array representing the grid, with rows and columns.
    pub fn get_grid(&self) -> &[[Token; 7]; 6] {
        &self.grid
    }

    /// Checks if a move to the specified column is valid.
    ///
    /// A move is valid if the column index is within bounds (0-6) and
    /// the top row of that column is empty.
    ///
    /// # Arguments
    /// * `column` - The column index (0-6)
    ///
    /// # Returns
    /// `true` if the move is valid, `false` otherwise.
    pub fn is_valid_move(&self, column: usize) -> bool {
        if column >= 7 {
            return false;
        }
        self.grid[5][column] == Token::Empty
    }
}

impl GameBoard for ConnectFour {
    type MoveType = usize;
    type PlayerType = Token;

    /// Returns the current player (1 for Red, 2 for Yellow).
    ///
    /// Determines the current player by counting tokens. Player 1 (Red) goes first.
    fn get_current_player(&self) -> Token {
        let mut red_count = 0;
        let mut yellow_count = 0;

        for row in 0..6 {
            for col in 0..7 {
                match self.grid[row][col] {
                    Token::Red => red_count += 1,
                    Token::Yellow => yellow_count += 1,
                    Token::Empty => {}
                }
            }
        }

        if red_count <= yellow_count {
            Token::Red // Player Red's turn
        } else {
            Token::Yellow // Player Yellow's turn
        }
    }

    /// Returns all columns where a token can be dropped.
    ///
    /// A column is available if its top row (row 5) is empty.
    fn get_available_moves(&self) -> Vec<Self::MoveType> {
        let mut moves = Vec::new();
        for col in 0..7 {
            if self.grid[5][col] == Token::Empty {
                moves.push(col);
            }
        }
        moves
    }

    /// Drops a token into the specified column.
    ///
    /// The token falls to the lowest available position in the column.
    ///
    /// # Arguments
    /// * `mv` - The column index (0-6) to drop the token into
    /// * `player` - The player making the move (1 for Red, 2 for Yellow)
    ///
    /// # Errors
    /// Returns an error if:
    /// - The column index is out of bounds (>= 7)
    /// - The column is full (top row is not empty)
    fn play(&mut self, mv: Self::MoveType, token: Token) -> Result<(), String> {
        if mv >= 7 || self.grid[5][mv] != Token::Empty {
            return Err("Invalid move".to_string());
        }

        for row in 0..6 {
            if self.grid[row][mv] == Token::Empty {
                self.grid[row][mv] = token;
                break;
            }
        }

        Ok(())
    }

    /// Returns the current status of the game.
    ///
    /// Checks for four connected tokens in any direction (horizontal, vertical, or diagonal).
    /// Returns `BoardStatus::Draw` if the board is full with no winner.
    fn get_status(&self) -> BoardStatus {
        let mut status = BoardStatus::InProgress;

        // Check columns for 4 in a row
        for col in 0..7 {
            for row in 0..3 {
                if self.grid[row][col] != Token::Empty
                    && self.grid[row][col] == self.grid[row + 1][col]
                    && self.grid[row][col] == self.grid[row + 2][col]
                    && self.grid[row][col] == self.grid[row + 3][col]
                {
                    return BoardStatus::Win(self.grid[row][col].into());
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
                    return BoardStatus::Win(self.grid[row][col].into());
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
                    return BoardStatus::Win(self.grid[row][col].into());
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
                    return BoardStatus::Win(self.grid[row][col].into());
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

impl fmt::Display for ConnectFour {
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
