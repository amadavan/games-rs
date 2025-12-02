use crate::{
    GameBoard,
    agents::ScoreFunction,
    connect_four::{ConnectFour, Token},
    ultimate_ttt::UltimateTTT,
};

pub struct NaiveScorer<Game: GameBoard> {
    _marker: std::marker::PhantomData<Game>,
}

impl<Game: GameBoard> NaiveScorer<Game> {
    /// Creates a new NaiveScorer agent for Connect Four.
    pub fn new() -> Self {
        NaiveScorer {
            _marker: std::marker::PhantomData,
        }
    }
}

impl ScoreFunction<ConnectFour> for NaiveScorer<ConnectFour> {
    /// Scores the given game board using a naive heuristic.
    ///
    /// # Arguments
    /// * `board` - The current game board state
    /// * `move` - The move being considered
    /// * `player` - The player who is making the move
    ///
    /// # Returns
    /// A floating-point score representing the desirability of the board state.
    fn score(
        &self,
        board: &ConnectFour,
        mv: &<ConnectFour as GameBoard>::MoveType,
        player: u8,
    ) -> f32 {
        let mut next_board = board.clone();
        next_board.play(*mv, player).unwrap();
        let grid = next_board.get_grid();

        let mut score = 0.0;

        // Only consider the played move when assessing personal score
        let col = mv;
        let row = (0..6)
            .rev()
            .find(|&row| grid[row][*col] != Token::Empty)
            .unwrap_or(0);

        // Check for wins
        for c in 0..4 {
            if grid[row][c] != Token::Empty
                && grid[row][c] == grid[row][c + 1]
                && grid[row][c] == grid[row][c + 2]
                && grid[row][c] == grid[row][c + 3]
            {
                if grid[row][c] != player.into() {
                    score -= 120.0;
                } else if col >= &c && col <= &(c + 3) {
                    score += 100.0
                }
            }
        }

        for r in 0..3 {
            if grid[r][*col] != Token::Empty
                && grid[r][*col] == grid[r + 1][*col]
                && grid[r][*col] == grid[r + 2][*col]
                && grid[r][*col] == grid[r + 3][*col]
            {
                if grid[r][*col] != player.into() {
                    score -= 120.0;
                } else if row >= r && row <= r + 3 {
                    score += 100.0
                }
            }
        }

        for r in 0..3 {
            for c in 0..4 {
                if grid[r][c] != Token::Empty
                    && grid[r][c] == grid[r + 1][c + 1]
                    && grid[r][c] == grid[r + 2][c + 2]
                    && grid[r][c] == grid[r + 3][c + 3]
                {
                    if grid[r][c] != player.into() {
                        score -= 120.0;
                    } else if row >= r && row <= r + 3 && col >= &c && col <= &(c + 3) {
                        score += 100.0
                    }
                }
            }
        }

        for r in 0..3 {
            for c in 3..7 {
                if grid[r][c] != Token::Empty
                    && grid[r][c] == grid[r + 1][c - 1]
                    && grid[r][c] == grid[r + 2][c - 2]
                    && grid[r][c] == grid[r + 3][c - 3]
                {
                    if grid[r][c] != player.into() {
                        score -= 120.0;
                    } else if row >= r && row <= r + 3 && col >= &(c - 3) && col <= &c {
                        score += 100.0;
                    }
                }
            }
        }

        // Check for 3-in-a-rows
        for c in 0..5 {
            if grid[row][c] != Token::Empty
                && grid[row][c] == grid[row][c + 1]
                && grid[row][c] == grid[row][c + 2]
            {
                if grid[row][c] != player.into() {
                    score -= 12.0;
                } else if col >= &c && col <= &(c + 2) {
                    score += 10.0
                }
            }
        }

        for r in 0..4 {
            if grid[r][*col] != Token::Empty
                && grid[r][*col] == grid[r + 1][*col]
                && grid[r][*col] == grid[r + 2][*col]
            {
                if grid[r][*col] != player.into() {
                    score -= 12.0;
                } else if row >= r && row <= r + 2 {
                    score += 10.0
                }
            }
        }

        for r in 0..4 {
            for c in 0..5 {
                if grid[r][c] != Token::Empty
                    && grid[r][c] == grid[r + 1][c + 1]
                    && grid[r][c] == grid[r + 2][c + 2]
                {
                    if grid[r][c] != player.into() {
                        score -= 12.0;
                    } else if row >= r && row <= r + 2 && col >= &c && col <= &(c + 2) {
                        score += 10.0
                    }
                }
            }
        }

        for r in 0..4 {
            for c in 2..7 {
                if grid[r][c] != Token::Empty
                    && grid[r][c] == grid[r + 1][c - 1]
                    && grid[r][c] == grid[r + 2][c - 2]
                {
                    if grid[r][c] != player.into() {
                        score -= 12.0;
                    } else if row >= r && row <= r + 2 && col >= &(c - 2) && col <= &c {
                        score += 10.0
                    }
                }
            }
        }

        // Check 2-in-a-rows
        for c in 0..6 {
            if grid[row][c] != Token::Empty && grid[row][c] == grid[row][c + 1] {
                if grid[row][c] != player.into() {
                    score -= 2.0;
                } else if col >= &c && col <= &(c + 1) {
                    score += 1.0
                }
            }
        }

        for r in 0..5 {
            if grid[r][*col] != Token::Empty && grid[r][*col] == grid[r + 1][*col] {
                if grid[r][*col] != player.into() {
                    score -= 2.0;
                } else if row >= r && row <= r + 1 {
                    score += 1.0
                }
            }
        }

        for r in 0..5 {
            for c in 0..6 {
                if grid[r][c] != Token::Empty && grid[r][c] == grid[r + 1][c + 1] {
                    if grid[r][c] != player.into() {
                        score -= 2.0;
                    } else if row >= r && row <= r + 1 && col >= &c && col <= &(c + 1) {
                        score += 1.0
                    }
                }
            }
        }

        for r in 0..5 {
            for c in 1..7 {
                if grid[r][c] != Token::Empty && grid[r][c] == grid[r + 1][c - 1] {
                    if grid[r][c] != player.into() {
                        score -= 2.0;
                    } else if row >= r && row <= r + 1 && col >= &(c - 1) && col <= &c {
                        score += 1.0
                    }
                }
            }
        }

        score
    }
}

impl ScoreFunction<UltimateTTT> for NaiveScorer<UltimateTTT> {
    /// Scores the given game board using a naive heuristic.
    ///
    /// # Arguments
    /// * `board` - The current game board state
    /// * `move` - The move being considered
    /// * `player` - The player who is making the move
    ///
    /// # Returns
    /// A floating-point score representing the desirability of the board state.
    fn score(
        &self,
        board: &UltimateTTT,
        mv: &<UltimateTTT as GameBoard>::MoveType,
        player: u8,
    ) -> f32 {
        // Implement a simple heuristic to score the board
        // This is a placeholder implementation
        0.0
    }
}
