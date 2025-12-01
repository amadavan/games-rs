pub mod monte_carlo_graph;

use rand::seq::IndexedRandom;

use crate::{BoardStatus, GameBoard, agents::monte_carlo_graph::MonteCarloGraph};

pub trait Agent<Game: GameBoard> {
    fn get_move(&self, board: &Game) -> <Game as GameBoard>::MoveType;

    fn notify(&mut self, _moves: &Vec<(u8, Game)>, _status: BoardStatus) -> () {
        ()
    }
}

pub struct PlayerAgent<Game: GameBoard> {
    pub player: u8,
    _marker: std::marker::PhantomData<Game>,
}

impl<Game: GameBoard> PlayerAgent<Game> {
    pub fn new(player: u8) -> Self {
        PlayerAgent {
            player,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<Game: GameBoard> Agent<Game> for PlayerAgent<Game> {
    fn get_move(&self, board: &Game) -> <Game as GameBoard>::MoveType {
        let mut mv = None;

        while mv.is_none() {
            println!("{:?}", board);
            println!("Player {}, enter your move:", self.player);

            // Get the user input
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let parsed_move: Result<<Game as GameBoard>::MoveType, _> = input
                .trim()
                .parse()
                .map_err(|_| "Failed to parse move".to_string());

            if board
                .get_available_moves()
                .contains(&parsed_move.as_ref().unwrap())
            {
                mv = Some(parsed_move.unwrap());
            } else {
                println!("Invalid move, try again.");
            }
        }

        mv.unwrap()
    }
}

pub struct RandomAgent<Game: GameBoard> {
    _marker: std::marker::PhantomData<Game>,
}

impl<Game: GameBoard> RandomAgent<Game> {
    pub fn new() -> Self {
        RandomAgent {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<Game: GameBoard> Agent<Game> for RandomAgent<Game> {
    fn get_move(&self, board: &Game) -> <Game as GameBoard>::MoveType {
        let available_moves = board.get_available_moves();
        let mut rng = rand::rng();
        available_moves.choose(&mut rng).unwrap().clone()
    }
}

pub struct MonteCarloGraphSearch<Game: GameBoard> {
    graph: MonteCarloGraph<Game>,
}

impl<Game: GameBoard> MonteCarloGraphSearch<Game> {
    pub fn new() -> Self {
        MonteCarloGraphSearch {
            graph: MonteCarloGraph::new(),
        }
    }

    pub fn from_graph(graph: MonteCarloGraph<Game>) -> Self {
        MonteCarloGraphSearch { graph }
    }
}

impl<Game: GameBoard> Agent<Game> for MonteCarloGraphSearch<Game> {
    fn get_move(&self, board: &Game) -> <Game as GameBoard>::MoveType {
        // Implementation of move selection using Monte Carlo Graph Search
        let available_moves = board.get_available_moves();

        let values = available_moves
            .iter()
            .map(|(mv)| {
                let mut next_board = board.clone();
                let _ = next_board.play(*mv, board.get_current_player());
                let edge_weight = self.graph.edge_weight(board.clone(), next_board.clone());
                if edge_weight.is_none() {
                    (mv, 1f64 + 2f64.sqrt())
                } else {
                    let edge_weight = edge_weight.unwrap();
                    let target_count = self
                        .graph
                        .edges_from(&next_board)
                        .iter()
                        .map(|(_, (_, n))| *n)
                        .sum::<usize>();
                    let w = (edge_weight.0 + 1) as f64;
                    let n = (edge_weight.1 + 1) as f64;
                    let N = (target_count + 1) as f64;
                    (mv, w / n + (2.0 * N.ln() / n).sqrt())
                }
            })
            .collect::<Vec<_>>();

        let mut ordered_values = values.into_iter().collect::<Vec<_>>();
        ordered_values.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let maximizers = ordered_values
            .iter()
            .filter(|(_, v)| *v < ordered_values[0].1 + 1e-6)
            .collect::<Vec<_>>();
        use rand::Rng;
        let mut random = rand::rng();
        let choice = maximizers[random.random_range(0..maximizers.len())];

        choice.0.clone()
    }

    fn notify(&mut self, _moves: &Vec<(u8, Game)>, _status: BoardStatus) -> () {
        // Update the Monte Carlo Graph based on the game outcome
        let path = _moves.iter().map(|(_, b)| b.clone()).collect();
        self.graph.back_propogate(path, _status);
    }
}
