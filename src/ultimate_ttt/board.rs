use std::fmt;
use std::hash::Hash;
use crate::ultimate_ttt::Player;

use serde::{Deserialize, Serialize};


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BoardStatus {
    InProgress,
    Won(Player),
    Draw,
}

#[derive(Clone, Copy, PartialEq)]
pub struct Move {
    microboard_row: usize,
    microboard_col: usize,
    cell_row: usize,
    cell_col: usize,
}

impl Move {
    pub fn new(
        microboard_row: usize,
        microboard_col: usize,
        cell_row: usize,
        cell_col: usize,
    ) -> Self {
        Move {
            microboard_row,
            microboard_col,
            cell_row,
            cell_col,
        }
    }

    pub fn get_microboard_position(&self) -> (usize, usize) {
        (self.microboard_row, self.microboard_col)
    }

    pub fn get_cell_position(&self) -> (usize, usize) {
        (self.cell_row, self.cell_col)
    }
}

impl From<(usize, usize, usize, usize)> for Move {
    fn from(coords: (usize, usize, usize, usize)) -> Self {
        Move {
            microboard_row: coords.0,
            microboard_col: coords.1,
            cell_row: coords.2,
            cell_col: coords.3,
        }
    }
}

impl From<Move> for (usize, usize, usize, usize) {
    fn from(mv: Move) -> (usize, usize, usize, usize) {
        (
            mv.microboard_row,
            mv.microboard_col,
            mv.cell_row,
            mv.cell_col,
        )
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({}, {}, {}, {})",
            self.microboard_row, self.microboard_col, self.cell_row, self.cell_col
        )
    }
}

#[derive(Clone, PartialEq)]
pub struct Board {
    previous_move: Option<Move>,
    status: BoardStatus,
    cells: Vec<Vec<MicroBoard>>,
}

impl Board {
    pub fn new() -> Self {
        Board {
            previous_move: None,
            status: BoardStatus::InProgress,
            cells: vec![vec![MicroBoard::new(); 3]; 3],
        }
    }

    pub fn set_status(&mut self, status: BoardStatus) {
        self.status = status;
    }

    pub fn get_status(&self) -> &BoardStatus {
        &self.status
    }

    pub fn get_cells(&self) -> &Vec<Vec<MicroBoard>> {
        &self.cells
    }

    pub fn get_microboard(&self, row: usize, col: usize) -> &MicroBoard {
        &self.cells[row][col]
    }

    pub fn update_status(&mut self) -> &BoardStatus {
        if self.is_won() {
            // Determine the winner
            for i in 0..3 {
                for j in 0..3 {
                    if self.cells[i][j].status != BoardStatus::InProgress {
                        self.status = self.cells[i][j].status.clone();
                    }
                }
            }
        } else if self.get_available_moves().is_empty() {
            self.status = BoardStatus::Draw;
        }
        &self.status
    }

    pub fn is_won(&self) -> bool {
        // Check rows and columns
        for i in 0..3 {
            if self.cells[i][0].status != BoardStatus::InProgress
                && self.cells[i][0].status == self.cells[i][1].status
                && self.cells[i][1].status == self.cells[i][2].status
            {
                return true;
            }
            if self.cells[0][i].status != BoardStatus::InProgress
                && self.cells[0][i].status == self.cells[1][i].status
                && self.cells[1][i].status == self.cells[2][i].status
            {
                return true;
            }
        }
        // Check diagonals
        if self.cells[0][0].status != BoardStatus::InProgress
            && self.cells[0][0].status == self.cells[1][1].status
            && self.cells[1][1].status == self.cells[2][2].status
        {
            return true;
        }
        if self.cells[0][2].status != BoardStatus::InProgress
            && self.cells[0][2].status == self.cells[1][1].status
            && self.cells[1][1].status == self.cells[2][0].status
        {
            return true;
        }
        false
    }

    pub fn get_available_moves(&self) -> Vec<Move> {
        // Check if the game is already won
        if self.is_won() {
            return Vec::new();
        }

        // Get set of available microboards
        let mut microboard_moves = Vec::new();

        // Only restrict to previous board if it's still in progress
        if let Some(prev_move) = self.previous_move {
            let (cell_row, cell_col) = prev_move.get_cell_position();
            let microboard = &self.cells[cell_row][cell_col];
            if microboard.status == BoardStatus::InProgress {
                microboard_moves.push((cell_row, cell_col));
            }
        }

        if microboard_moves.is_empty() {
            for i in 0..3 {
                for j in 0..3 {
                    if self.cells[i][j].status == BoardStatus::InProgress {
                        microboard_moves.push((i, j));
                    }
                }
            }
        }

        // Return the set of available moves
        let mut moves = Vec::new();
        for (microboard_row, microboard_col) in microboard_moves {
            let microboard = &self.cells[microboard_row][microboard_col];
            for (cell_row, cell_col) in microboard.get_available_moves() {
                moves.push((microboard_row, microboard_col, cell_row, cell_col).into());
            }
        }
        moves
    }

    pub fn is_valid_move(&self, mv: Move) -> bool {
        // Check if the game is already won
        if self.is_won() {
            return false;
        }

        self.get_available_moves().contains(&mv)
    }

    pub fn play(&mut self, mv: Move, player: Player) -> Result<(), String> {
        let (microboard_row, microboard_col) = mv.get_microboard_position();
        let (cell_row, cell_col) = mv.get_cell_position();

        // Check if move is valid
        if self.is_won() {
            return Err("Board already won".to_string());
        }
        if microboard_row >= 3 || microboard_col >= 3 {
            return Err("Invalid microboard position".to_string());
        }
        if !self.is_valid_move(mv) {
            println!(
                "Invalid move: previous_move = {:?}, attempted move = {:?}",
                self.previous_move, mv
            );
            println!("Valid moves: {:?}", self.get_available_moves());
            return Err("Invalid move based on previous move".to_string());
        }

        // Play the move on the specified microboard
        let microboard = &mut self.cells[microboard_row][microboard_col];
        microboard.play(cell_row, cell_col, player)?;

        // Set the previous move
        self.previous_move = Some(Move::new(
            microboard_row,
            microboard_col,
            cell_row,
            cell_col,
        ));

        // Update the overall board status
        self.update_status();

        Ok(())
    }

    pub fn to_hash(&self) -> [Player; 81] {
        let mut board_state = [Player::Empty; 81];
        for (mi, row) in self.cells.iter().enumerate() {
            for (mj, microboard) in row.iter().enumerate() {
                for (ci, microboard_row) in microboard.cells.iter().enumerate() {
                    for (cj, &cell) in microboard_row.iter().enumerate() {
                        let idx = mi * 27 + mj * 9 + ci * 3 + cj;
                        board_state[idx] = cell;
                    }
                }
            }
        }
        board_state
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "--------------------")?;
        for row in &self.cells {
            for microboard_row in 0..3 {
                write!(f, "|  ")?;
                for microboard in row {
                    for cell in 0..3 {
                        let cell_state: char = microboard.cells[microboard_row][cell].into();
                        write!(f, "{}", cell_state)?;
                    }
                    write!(f, " | ")?;
                }
                writeln!(f)?;
            }
            writeln!(f, "--------------------")?;
        }
        Ok(())
    }
}

impl Eq for Board {}

#[derive(Clone, Debug, PartialEq)]
pub struct MicroBoard {
    status: BoardStatus,
    cells: Vec<Vec<Player>>,
}

impl MicroBoard {
    pub fn new() -> Self {
        MicroBoard {
            status: BoardStatus::InProgress,
            cells: vec![vec![Player::Empty; 3]; 3],
        }
    }

    pub fn get_status(&self) -> &BoardStatus {
        &self.status
    }

    pub fn get_cells(&self) -> &Vec<Vec<Player>> {
        &self.cells
    }

    pub fn is_won(&self) -> bool {
        // Check rows and columns
        for i in 0..3 {
            if self.cells[i][0] != Player::Empty
                && self.cells[i][0] == self.cells[i][1]
                && self.cells[i][1] == self.cells[i][2]
            {
                return true;
            }
            if self.cells[0][i] != Player::Empty
                && self.cells[0][i] == self.cells[1][i]
                && self.cells[1][i] == self.cells[2][i]
            {
                return true;
            }
        }
        // Check diagonals
        if self.cells[0][0] != Player::Empty
            && self.cells[0][0] == self.cells[1][1]
            && self.cells[1][1] == self.cells[2][2]
        {
            return true;
        }
        if self.cells[0][2] != Player::Empty
            && self.cells[0][2] == self.cells[1][1]
            && self.cells[1][1] == self.cells[2][0]
        {
            return true;
        }
        false
    }

    fn get_available_moves(&self) -> Vec<(usize, usize)> {
        let mut moves = Vec::new();
        for i in 0..3 {
            for j in 0..3 {
                if self.cells[i][j] == Player::Empty {
                    moves.push((i, j));
                }
            }
        }
        moves
    }

    fn play(&mut self, row: usize, col: usize, player: Player) -> Result<(), String> {
        // Check if move is valid
        if self.is_won() {
            return Err("MicroBoard already won".to_string());
        }
        if self.cells[row][col] != Player::Empty {
            return Err("Cell already occupied".to_string());
        }

        // Play the move
        self.cells[row][col] = player;

        // Update status
        if self.is_won() {
            self.status = BoardStatus::Won(player);
        } else if self.get_available_moves().is_empty() {
            self.status = BoardStatus::Draw;
        }

        Ok(())
    }
}
