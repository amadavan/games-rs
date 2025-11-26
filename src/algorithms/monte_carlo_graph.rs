use std::hash::RandomState;

use petgraph::algo;
use petgraph::visit::EdgeRef;
use petgraph::{Direction::Outgoing, prelude::DiGraphMap};

pub struct MonteCarloGraph<N>
where
    N: std::hash::Hash + Eq + Clone + Copy + Ord + Default,
{
    // Graph structure and methods would be defined here
    graph: DiGraphMap<N, (usize, usize)>,
    root: N,
}

impl<N> MonteCarloGraph<N>
where
    N: std::hash::Hash + Eq + Clone + Copy + Ord + Default,
{
    pub fn new() -> Self {
        MonteCarloGraph {
            graph: DiGraphMap::new(),
            root: N::default(),
        }
    }

    pub fn get_node_aggregate_values(&self, node: &N) -> (usize, usize) {
        let mut total_wins = 0;
        let mut total_simulations = 0;

        for edge in self.graph.edges_directed(node.clone(), Outgoing) {
            let (wins, simulations) = edge.weight();
            total_wins += wins;
            total_simulations += simulations;
        }

        (total_wins, total_simulations)
    }

    pub fn contains_node(&self, n: &N) -> bool {
        self.graph.contains_node(*n)
    }

    pub fn contains_edge(&self, from: &N, to: &N) -> bool {
        self.graph.contains_edge(*from, *to)
    }

    pub fn back_propogate(&mut self, path: Vec<N>, win: bool) {
        // Make sure that the path is valid
        if path.len() < 2 {
            return;
        }

        // Implementation of backpropagation logic
        let current_node = path.last().unwrap();
        let previous_node = &path[path.len() - 2];

        // Check if we've been along this edge and ignore if so
        if let Some((_, simulations)) = self.graph.edge_weight(*previous_node, *current_node) {
            if *simulations > 0 {
                return;
            }
        }

        // Check if the node exists and set the weight accordingly
        let weight = if self.graph.contains_node(*current_node) {
            self.get_node_aggregate_values(current_node)
        } else {
            self.graph.add_node(*current_node);
            if win { (1, 1) } else { (0, 1) }
        };

        // Add the edge to the graph
        self.graph.add_edge(*previous_node, *current_node, weight);

        // Get all simple paths from the above node to the root node
        // Update them based on the weight of the current edge
        let paths = algo::all_simple_paths::<Vec<_>, _, RandomState>(
            &self.graph,
            *previous_node,
            self.root,
            0,
            None,
        )
        .collect::<Vec<_>>();

        // If it is the current player then we need to update with the number of wins, otherwise with the difference
        for p in paths.iter() {
            for (l, i) in (0..p.len() - 1).enumerate() {
                let edge_weight = self.graph.edge_weight_mut(p[i], p[i + 1]).unwrap();
                edge_weight.0 += if l % 2 == 0 {
                    weight.0
                } else {
                    weight.1 - weight.0
                };
                edge_weight.1 += weight.1;
            }
        }
    }
}
