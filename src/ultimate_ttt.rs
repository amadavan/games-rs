use std::fmt::Debug;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::BoardStatus;
use crate::GameBoard;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
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
    fn from(t: (u8, u8, u8, u8)) -> Self {
        Move {
            microboard_row: t.0,
            microboard_col: t.1,
            cell_row: t.2,
            cell_col: t.3,
        }
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct UltimateTTT {
    boards: [[MicroBoard; 3]; 3],
    next_microboard: Option<(u8, u8)>,
}

impl UltimateTTT {
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

    fn get_current_player(&self) -> u8 {
        let mut x_count = 0;
        let mut o_count = 0;

        for i in 0..3 {
            for j in 0..3 {
                for row in 0..3 {
                    for col in 0..3 {
                        match self.boards[i][j].grid[row][col] {
                            1 => x_count += 1,
                            2 => o_count += 1,
                            _ => {}
                        }
                    }
                }
            }
        }

        if x_count <= o_count {
            1 // Player X's turn
        } else {
            2 // Player O's turn
        }
    }

    fn get_available_moves(&self) -> Vec<Self::MoveType> {
        // Implementation to get available moves
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

    fn play(&mut self, mv: Self::MoveType, player: impl Into<u8>) -> Result<(), String> {
        // Implementation to play a move
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

    fn get_status(&self) -> BoardStatus {
        // Implementation to determine the current status of the ultimate board
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
                            1 => 'X',
                            2 => 'O',
                            _ => '-',
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

#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct MicroBoard {
    grid: [[u8; 3]; 3],
}

impl MicroBoard {
    pub fn new() -> Self {
        MicroBoard { grid: [[0; 3]; 3] }
    }

    pub fn get_status(&self) -> BoardStatus {
        // Implementation to determine the current status of the micro board

        // Check rows and columns for win
        for i in 0..3 {
            if self.grid[i][0] != 0
                && self.grid[i][0] == self.grid[i][1]
                && self.grid[i][1] == self.grid[i][2]
            {
                return BoardStatus::Win(self.grid[i][0]);
            }
            if self.grid[0][i] != 0
                && self.grid[0][i] == self.grid[1][i]
                && self.grid[1][i] == self.grid[2][i]
            {
                return BoardStatus::Win(self.grid[0][i]);
            }
        }
        // Check diagonals
        if self.grid[0][0] != 0
            && self.grid[0][0] == self.grid[1][1]
            && self.grid[1][1] == self.grid[2][2]
        {
            return BoardStatus::Win(self.grid[0][0]);
        }
        if self.grid[0][2] != 0
            && self.grid[0][2] == self.grid[1][1]
            && self.grid[1][1] == self.grid[2][0]
        {
            return BoardStatus::Win(self.grid[0][2]);
        }

        if self.get_available_moves().is_empty() {
            return BoardStatus::Draw;
        }

        BoardStatus::InProgress
    }

    pub fn get_available_moves(&self) -> Vec<(u8, u8)> {
        let mut moves = Vec::new();
        for i in 0..3 {
            for j in 0..3 {
                if self.grid[i][j] == 0 {
                    moves.push((i as u8, j as u8));
                }
            }
        }
        moves
    }

    pub fn play(&mut self, row: u8, col: u8, player: impl Into<u8>) -> Result<(), String> {
        let p = player.into();
        if self.grid[row as usize][col as usize] != 0 {
            return Err("Cell already occupied".to_string());
        }
        self.grid[row as usize][col as usize] = p;
        Ok(())
    }
}
