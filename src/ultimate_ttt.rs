//! Ultimate Tic-Tac-Toe game implementation.
//!
//! This module implements Ultimate Tic-Tac-Toe, a variant where the game board consists
//! of a 3×3 grid of standard Tic-Tac-Toe boards. Players must win individual boards to
//! claim positions in the outer board, and the first player to win three outer boards
//! in a row wins the game.

use std::fmt::Debug;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::BoardStatus;
use crate::GameBoard;

use derive_aliases::derive;

#[derive(..StdTraits, Serialize, Deserialize, Debug)]
pub enum Player {
    X,
    O,
    Empty,
}

impl From<u8> for Player {
    fn from(value: u8) -> Self {
        match value {
            1 => Player::X,
            2 => Player::O,
            _ => Player::Empty,
        }
    }
}

impl Into<u8> for Player {
    fn into(self) -> u8 {
        match self {
            Player::X => 1,
            Player::O => 2,
            Player::Empty => 0,
        }
    }
}

/// Represents a move in Ultimate Tic-Tac-Toe.
///
/// A move specifies both which microboard to play in (via `microboard_row` and `microboard_col`)
/// and which cell within that microboard (via `cell_row` and `cell_col`).
#[derive(..StdTraits, Serialize, Deserialize)]
pub struct Move {
    microboard_row: u8,
    microboard_col: u8,
    cell_row: u8,
    cell_col: u8,
}

impl Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Microboard: ({}, {}), Cell: ({}, {})",
            self.microboard_row, self.microboard_col, self.cell_row, self.cell_col
        )
    }
}

impl FromStr for Move {
    type Err = String;

    /// Parses a move from a string containing four space-separated numbers.
    ///
    /// # Format
    /// The string should contain: `microboard_row microboard_col cell_row cell_col`
    ///
    /// # Examples
    /// ```ignore
    /// let move_str = "1 2 0 1";
    /// let mv: Move = move_str.parse()?;
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.trim().split_whitespace().collect();
        if parts.len() != 4 {
            return Err("Input must contain exactly four numbers".to_string());
        }

        let microboard_row = parts[0].parse::<u8>().map_err(|_| "Invalid number")?;
        let microboard_col = parts[1].parse::<u8>().map_err(|_| "Invalid number")?;
        let cell_row = parts[2].parse::<u8>().map_err(|_| "Invalid number")?;
        let cell_col = parts[3].parse::<u8>().map_err(|_| "Invalid number")?;

        Ok(Move {
            microboard_row,
            microboard_col,
            cell_row,
            cell_col,
        })
    }
}

impl From<(u8, u8, u8, u8)> for Move {
    /// Creates a move from a tuple of (microboard_row, microboard_col, cell_row, cell_col).
    fn from(t: (u8, u8, u8, u8)) -> Self {
        Move {
            microboard_row: t.0,
            microboard_col: t.1,
            cell_row: t.2,
            cell_col: t.3,
        }
    }
}

/// The main Ultimate Tic-Tac-Toe game board.
///
/// This structure represents a 3×3 grid of microboards. The game follows these rules:
/// - Players alternate turns placing marks in individual cells
/// - The cell's position determines which microboard the next player must play in
/// - If a microboard is already won or full, the player can choose any available microboard
/// - A player wins by getting three microboards in a row (horizontally, vertically, or diagonally)
#[derive(..StdTraits, Serialize, Deserialize)]
pub struct UltimateTTT {
    boards: [[MicroBoard; 3]; 3],
    /// The microboard where the next move must be played, or None if any board is allowed.
    next_microboard: Option<(u8, u8)>,
}

impl UltimateTTT {
    /// Creates a new Ultimate Tic-Tac-Toe game with all boards empty.
    pub fn new() -> Self {
        UltimateTTT {
            boards: [
                [MicroBoard::new(), MicroBoard::new(), MicroBoard::new()],
                [MicroBoard::new(), MicroBoard::new(), MicroBoard::new()],
                [MicroBoard::new(), MicroBoard::new(), MicroBoard::new()],
            ],
            next_microboard: None,
        }
    }
}

impl GameBoard for UltimateTTT {
    type MoveType = Move;
    type PlayerType = Player;

    /// Returns the current player (1 for X, 2 for O).
    ///
    /// Determines the current player by counting moves. Player 1 (X) goes first.
    fn get_current_player(&self) -> Player {
        let mut x_count = 0;
        let mut o_count = 0;

        for i in 0..3 {
            for j in 0..3 {
                for row in 0..3 {
                    for col in 0..3 {
                        match self.boards[i][j].grid[row][col] {
                            Player::X => x_count += 1,
                            Player::O => o_count += 1,
                            Player::Empty => {}
                        }
                    }
                }
            }
        }

        if x_count <= o_count {
            Player::X // Player X's turn
        } else {
            Player::O // Player O's turn
        }
    }

    /// Returns all valid moves for the current game state.
    ///
    /// If a previous move directed play to a specific microboard (and that board is still playable),
    /// only moves in that microboard are returned. Otherwise, moves from all playable microboards
    /// are returned.
    fn get_available_moves(&self) -> Vec<Self::MoveType> {
        let mut available_microboards = Vec::new();
        let mut available_moves = Vec::new();

        if let Some((row, col)) = self.next_microboard {
            available_microboards.push((row, col));
        } else {
            for i in 0..3 {
                for j in 0..3 {
                    if self.boards[i][j].get_status() == BoardStatus::InProgress {
                        available_microboards.push((i as u8, j as u8));
                    }
                }
            }
        }

        for (microboard_row, microboard_col) in available_microboards {
            let microboard = &self.boards[microboard_row as usize][microboard_col as usize];
            let moves = microboard.get_available_moves();
            for (cell_row, cell_col) in moves {
                available_moves.push(Move::from((
                    microboard_row,
                    microboard_col,
                    cell_row,
                    cell_col,
                )));
            }
        }

        available_moves
    }

    /// Plays a move on the board.
    ///
    /// # Arguments
    /// * `mv` - The move to play
    /// * `player` - The player making the move (1 or 2)
    ///
    /// # Errors
    /// Returns an error if:
    /// - The game is already over
    /// - The move is not in the list of available moves
    /// - The target cell is already occupied
    ///
    /// # Side Effects
    /// Updates `next_microboard` to direct the next player to the appropriate board.
    fn play(&mut self, mv: Self::MoveType, player: Self::PlayerType) -> Result<(), String> {
        if self.get_status() != BoardStatus::InProgress {
            return Err("Game is already over".to_string());
        }

        if !self.get_available_moves().contains(&mv) {
            return Err("Invalid move".to_string());
        }

        // Play the move on the specified microboard
        let (microboard_row, microboard_col, cell_row, cell_col) = (
            mv.microboard_row,
            mv.microboard_col,
            mv.cell_row,
            mv.cell_col,
        );
        let microboard = &mut self.boards[microboard_row as usize][microboard_col as usize];
        microboard.play(cell_row, cell_col, player)?;

        // Set the previous move
        self.next_microboard = Some((cell_row, cell_col));

        Ok(())
    }

    /// Returns the current status of the game.
    ///
    /// Checks for wins by examining if three microboards in a row have been won by the same player.
    /// Returns `BoardStatus::Draw` if no moves are available and no player has won.
    fn get_status(&self) -> BoardStatus {
        // Check rows and columns
        for i in 0..3 {
            if self.boards[i][0].get_status() != BoardStatus::InProgress
                && self.boards[i][0].get_status() == self.boards[i][1].get_status()
                && self.boards[i][1].get_status() == self.boards[i][2].get_status()
            {
                return self.boards[i][0].get_status();
            }
            if self.boards[0][i].get_status() != BoardStatus::InProgress
                && self.boards[0][i].get_status() == self.boards[1][i].get_status()
                && self.boards[1][i].get_status() == self.boards[2][i].get_status()
            {
                return self.boards[0][i].get_status();
            }
        }
        // Check diagonals
        if self.boards[0][0].get_status() != BoardStatus::InProgress
            && self.boards[0][0].get_status() == self.boards[1][1].get_status()
            && self.boards[1][1].get_status() == self.boards[2][2].get_status()
        {
            return self.boards[0][0].get_status();
        }
        if self.boards[0][2].get_status() != BoardStatus::InProgress
            && self.boards[0][2].get_status() == self.boards[1][1].get_status()
            && self.boards[1][1].get_status() == self.boards[2][0].get_status()
        {
            return self.boards[0][2].get_status();
        }

        if self.get_available_moves().is_empty() {
            return BoardStatus::Draw;
        }

        BoardStatus::InProgress
    }
}

impl Default for UltimateTTT {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for UltimateTTT {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..3 {
            for row in 0..3 {
                for j in 0..3 {
                    for col in 0..3 {
                        let cell = self.boards[i][j].grid[row][col];
                        let symbol = match cell {
                            Player::X => 'X',
                            Player::O => 'O',
                            Player::Empty => '-',
                        };
                        write!(f, "{} ", symbol)?;
                    }
                    write!(f, "  ")?;
                }
                writeln!(f)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

/// A single 3×3 Tic-Tac-Toe board within the Ultimate Tic-Tac-Toe game.
///
/// Each cell can be empty (0), occupied by player 1 (X), or occupied by player 2 (O).
#[derive(..StdTraits, Serialize, Deserialize)]
pub struct MicroBoard {
    grid: [[Player; 3]; 3],
}

impl MicroBoard {
    /// Creates a new empty microboard.
    pub fn new() -> Self {
        MicroBoard {
            grid: [[Player::Empty; 3]; 3],
        }
    }

    /// Returns the current status of this microboard.
    ///
    /// Checks for wins (three in a row) and returns the winning player.
    /// Returns `BoardStatus::Draw` if the board is full with no winner.
    pub fn get_status(&self) -> BoardStatus {
        // Check rows and columns for win
        for i in 0..3 {
            if self.grid[i][0] != Player::Empty
                && self.grid[i][0] == self.grid[i][1]
                && self.grid[i][1] == self.grid[i][2]
            {
                return BoardStatus::Win(self.grid[i][0].into());
            }
            if self.grid[0][i] != Player::Empty
                && self.grid[0][i] == self.grid[1][i]
                && self.grid[1][i] == self.grid[2][i]
            {
                return BoardStatus::Win(self.grid[0][i].into());
            }
        }
        // Check diagonals
        if self.grid[0][0] != Player::Empty
            && self.grid[0][0] == self.grid[1][1]
            && self.grid[1][1] == self.grid[2][2]
        {
            return BoardStatus::Win(self.grid[0][0].into());
        }
        if self.grid[0][2] != Player::Empty
            && self.grid[0][2] == self.grid[1][1]
            && self.grid[1][1] == self.grid[2][0]
        {
            return BoardStatus::Win(self.grid[0][2].into());
        }

        if self.get_available_moves().is_empty() {
            return BoardStatus::Draw;
        }

        BoardStatus::InProgress
    }

    /// Returns all empty cells in this microboard as (row, col) tuples.
    pub fn get_available_moves(&self) -> Vec<(u8, u8)> {
        let mut moves = Vec::new();
        for i in 0..3 {
            for j in 0..3 {
                if self.grid[i][j] == Player::Empty {
                    moves.push((i as u8, j as u8));
                }
            }
        }
        moves
    }

    /// Places a player's mark in the specified cell.
    ///
    /// # Arguments
    /// * `row` - The row index (0-2)
    /// * `col` - The column index (0-2)
    /// * `player` - The player number (1 or 2)
    ///
    /// # Errors
    /// Returns an error if the cell is already occupied.
    pub fn play(&mut self, row: u8, col: u8, player: Player) -> Result<(), String> {
        if self.grid[row as usize][col as usize] != Player::Empty {
            return Err("Cell already occupied".to_string());
        }
        self.grid[row as usize][col as usize] = player;
        Ok(())
    }
}
