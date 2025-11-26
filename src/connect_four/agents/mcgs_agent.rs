use crate::algorithms::monte_carlo_graph::MonteCarloGraph;
use crate::connect_four::agents::Agent;
use crate::connect_four::board::{Board, BoardStatus, Token};

pub struct MonteCarloGraphSearch<'a> {
    // Fields and methods for the Monte Carlo Graph Search Agent
    token: Token,
    graph_data: &'a mut MonteCarloGraph<Board>,
}

impl<'a> MonteCarloGraphSearch<'a> {
    pub fn new(token: Token, graph_data: &'a mut MonteCarloGraph<Board>) -> Self {
        MonteCarloGraphSearch { token, graph_data }
    }
}

impl<'a> Agent for MonteCarloGraphSearch<'a> {
    fn get_move(&mut self, board: &Board, prev_moves: &Vec<(Token, Board)>) -> usize {
        // Implementation of move selection using Monte Carlo Graph Search
        let available_moves = board.get_available_moves();

        // Add all next nodes to the graph and back-propogate (only if the node has data)
        available_moves
            .iter()
            .enumerate()
            .filter(|(_, v)| **v)
            .for_each(|(i, _)| {
                let mut next_board = board.clone();
                let _ = next_board.play(i, self.token);
                self.graph_data.back_propogate(
                    prev_moves
                        .iter()
                        .cloned()
                        .chain(std::iter::once((self.token, next_board.clone())))
                        .map(|(_, b)| b)
                        .collect(),
                    false,
                );
            });

        let values = available_moves
            .iter()
            .enumerate()
            .filter(|(_, v)| **v)
            .map(|(i, _)| {
                let mut next_board = board.clone();
                let _ = next_board.play(i, self.token);
                let edge = self
                    .graph_data
                    .edge_weight(board.clone(), next_board.clone())
                    .unwrap();
                let v1 = self.graph_data.get_node_aggregate_values(&board.clone());
                let v2 = self
                    .graph_data
                    .get_node_aggregate_values(&next_board.clone());
                let w = (v2.0 + 1) as f64;
                let n = (v2.1 + 1) as f64;
                let N = (v1.1 + 1) as f64;
                (i, w / n + (2.0 * (N.ln() / n).sqrt()))
            })
            .collect::<Vec<_>>();

        // Return the result that maximimizes value
        values
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap()
            .0
    }

    fn notify_win(&mut self, moves: &Vec<(Token, Board)>, _status: BoardStatus) -> () {
        let path = moves.iter().map(|(_, b)| b.clone()).collect();
        self.graph_data.back_propogate(path, true);
    }
}
