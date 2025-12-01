//! Monte Carlo Graph Search data structure for game tree exploration.
//!
//! This module implements a graph-based approach to Monte Carlo tree search,
//! where game states are nodes and transitions are edges weighted with
//! win/simulation statistics.

use std::collections::HashSet;

use petgraph::Direction;
use petgraph::prelude::DiGraphMap;
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};

use crate::BoardStatus;

/// Game state classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// A player has won the game
    Win,
    /// The game ended in a draw
    Draw,
    /// The game is still ongoing
    InProgress,
}

/// Monte Carlo tree/graph search structure for game state exploration.
///
/// Tracks game states (nodes) and transitions (edges) with win/simulation statistics.
/// Edge weights are (wins, simulations) tuples where wins are counted from the parent's perspective.
#[derive(Serialize, Deserialize, Clone)]
#[serde(
    bound(serialize = "N: Serialize"),
    bound(deserialize = "N: for<'a> Deserialize<'a>")
)]
pub struct MonteCarloGraph<N>
where
    N: std::hash::Hash + Eq + Clone + Copy + Ord + Default + std::fmt::Debug,
{
    /// Directed graph: nodes are game states, edges are (wins, simulations)
    graph: DiGraphMap<N, (usize, usize)>,
    /// Root node representing the initial game state
    root: N,
}

impl<N> MonteCarloGraph<N>
where
    N: std::hash::Hash + Eq + Clone + Copy + Ord + Default + std::fmt::Debug + Serialize,
    for<'a> N: Deserialize<'a>,
{
    /// Creates a new graph with the default node as root.
    pub fn new() -> Self {
        let mut graph = DiGraphMap::new();
        graph.add_node(N::default());
        MonteCarloGraph {
            graph,
            root: N::default(),
        }
    }

    /// Aggregates statistics from all child nodes.
    ///
    /// Returns (wins, simulations) from the node's perspective by inverting child edge weights.
    pub fn get_node_aggregate_values(&self, node: &N) -> (usize, usize) {
        let (total_wins, total_simulations) = self
            .edges_from(node)
            .iter()
            .map(|(_, (wins, simulations))| (simulations - wins, *simulations))
            .fold((0, 0), |acc, x| (acc.0 + x.0, acc.1 + x.1));

        // TODO: this filter makes no sense...

        (total_wins, total_simulations)
    }

    /// Returns all nodes in the graph.
    pub fn nodes(&self) -> Vec<N> {
        self.graph.nodes().collect()
    }

    /// Checks if a node exists in the graph.
    pub fn contains_node(&self, n: &N) -> bool {
        self.graph.contains_node(*n)
    }

    /// Returns all incoming edges to a node with their weights.
    ///
    /// # Arguments
    /// * `n` - The target node
    ///
    /// # Returns
    /// A vector of (source_node, (wins, simulations)) tuples.
    pub fn edges_to(&self, n: &N) -> Vec<(N, (usize, usize))> {
        self.graph
            .edges_directed(*n, Direction::Incoming)
            .map(|e| (e.source(), *e.weight()))
            .collect()
    }

    /// Returns all outgoing edges from a node with their weights.
    ///
    /// # Arguments
    /// * `n` - The source node
    ///
    /// # Returns
    /// A vector of (target_node, (wins, simulations)) tuples.
    pub fn edges_from(&self, n: &N) -> Vec<(N, (usize, usize))> {
        self.graph
            .edges_directed(*n, Direction::Outgoing)
            .map(|e| (e.target(), *e.weight()))
            .collect()
    }

    /// Checks if an edge exists between two nodes.
    pub fn contains_edge(&self, from: &N, to: &N) -> bool {
        self.graph.contains_edge(*from, *to)
    }

    /// Returns the (wins, simulations) weight of an edge, or None if it doesn't exist.
    pub fn edge_weight(&self, from: N, to: N) -> Option<&(usize, usize)> {
        self.graph.edge_weight(from, to)
    }

    /// Propagates game outcome statistics backward through a path in the graph.
    ///
    /// Updates edge weights along the path based on the game's final outcome.
    /// Win/loss statistics are tracked from each node's perspective (wins are
    /// counted for the player who just moved to reach that state).
    ///
    /// # Arguments
    /// * `path` - The sequence of game states from start to end
    /// * `state` - The final game status (Win, Draw, or InProgress)
    ///
    /// # Behavior
    /// - Adds missing edges with (0,0) weight
    /// - Sets terminal node edge to (1,1) for wins, (0,1) for draws
    /// - Propagates aggregate statistics backward through parent nodes
    pub fn back_propogate(&mut self, path: Vec<N>, state: BoardStatus) {
        if path.len() < 2 {
            return;
        }

        let current_node = path.last().unwrap();
        let previous_node = &path[path.len() - 2];

        let current_weight = match state {
            BoardStatus::Win(_) => (1, 1),
            BoardStatus::Draw => (0, 1),
            BoardStatus::InProgress => (0, 0),
        };

        // If the leaf already exists, skip adding
        if self.graph.contains_edge(*previous_node, *current_node) {
            // Adjust weight if at a terminal node
            if state != BoardStatus::InProgress {
                let edge_weight = self
                    .graph
                    .edge_weight_mut(*previous_node, *current_node)
                    .unwrap();
                edge_weight.0 = current_weight.0;
                edge_weight.1 = current_weight.1;
            }
        } else {
            let weight = match state {
                BoardStatus::Win(_) => (1, 1),
                BoardStatus::Draw => (0, 1),
                BoardStatus::InProgress => (0, 0),
            };
            self.graph.add_edge(*previous_node, *current_node, weight);
        }

        // Add all edges that don't exist
        for i in (1..path.len()).rev() {
            let from = &path[i - 1];
            let to = &path[i];
            if !self.graph.contains_edge(*from, *to) {
                self.graph.add_edge(*from, *to, (0, 0));
            }
        }

        // Back-propogate through the tree
        let mut target_nodes = HashSet::new();
        target_nodes.insert(*previous_node);
        for _layer in 0..path.len() {
            let mut next_nodes = HashSet::new();
            for target_node in &target_nodes {
                let incoming_edges = self.edges_to(target_node);

                for (source, _) in incoming_edges {
                    let target_agg = self.get_node_aggregate_values(&target_node);
                    let edge_weight = self.graph.edge_weight_mut(source, *target_node).unwrap();
                    edge_weight.0 = target_agg.0;
                    edge_weight.1 = target_agg.1;

                    next_nodes.insert(source);
                }
            }

            target_nodes = next_nodes;
        }
    }

    /// Validates graph integrity.
    ///
    /// Checks that each non-leaf node's incoming edges match its aggregated outgoing edges.
    pub fn validate(&self) -> bool {
        let mut valid = true;
        self.graph
            .nodes()
            .filter(|n| self.graph.edges_directed(*n, Direction::Outgoing).count() > 0)
            .for_each(|n| {
                let node_weight = self.get_node_aggregate_values(&n);
                self.graph
                    .edges_directed(n, Direction::Incoming)
                    .for_each(|(_, _, weight)| valid &= *weight == node_weight);
            });
        valid
    }

    /// Serializes the graph to a file using bincode.
    pub fn to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let serialized = bincode::serde::encode_to_vec(&self, bincode::config::legacy())?;
        std::fs::write(path, serialized)?;
        Ok(())
    }

    /// Deserializes the graph from a bincode file.
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let data = std::fs::read(path)?;
        let deserialized: Self =
            bincode::serde::decode_from_slice(&data, bincode::config::legacy())?.0;
        Ok(deserialized)
    }
}

mod test {
    #[test]
    fn test_monte_carlo_graph_serialization() {
        use super::MonteCarloGraph;
        let mut mcg: MonteCarloGraph<u32> = MonteCarloGraph::new();
        mcg.graph.add_node(1);
        mcg.graph.add_node(2);
        mcg.graph.add_node(5);
        mcg.graph.add_edge(0, 1, (1, 4));
        mcg.graph.add_edge(1, 2, (3, 5));

        assert!(mcg.contains_node(&0));
        assert!(mcg.contains_node(&1));
        assert!(mcg.contains_node(&2));
        assert!(mcg.contains_node(&5));
        assert!(mcg.contains_edge(&0, &1));
        assert!(mcg.contains_edge(&1, &2));
        assert_eq!(mcg.edge_weight(0, 1), Some(&(1, 4)));
        assert_eq!(mcg.edge_weight(1, 2), Some(&(3, 5)));

        let path = "test_mcg.bin";
        mcg.to_file(path).unwrap();

        let loaded_mcg = MonteCarloGraph::from_file(path).unwrap();

        assert!(loaded_mcg.contains_node(&0));
        assert!(loaded_mcg.contains_node(&1));
        assert!(loaded_mcg.contains_node(&2));
        assert!(loaded_mcg.contains_edge(&0, &1));
        assert!(loaded_mcg.contains_edge(&1, &2));
        assert_eq!(loaded_mcg.edge_weight(0, 1), Some(&(1, 4)));
        assert_eq!(loaded_mcg.edge_weight(1, 2), Some(&(3, 5)));

        std::fs::remove_file(path).unwrap();
    }

    // #[test]
    // fn test_back_propogate() {
    //     use super::MonteCarloGraph;
    //     let mut mcg: MonteCarloGraph<u32> = MonteCarloGraph::new();
    //     let mut path = vec![0, 1];
    //     mcg.back_propogate(path.clone(), false);

    //     assert!(mcg.contains_node(&0));
    //     assert!(mcg.contains_node(&1));
    //     assert!(mcg.contains_edge(&0, &1));
    //     assert_eq!(mcg.edge_weight(0, 1), Some(&(0, 0)));

    //     path.push(2);
    //     mcg.back_propogate(path.clone(), true);

    //     assert!(mcg.contains_node(&2));
    //     assert!(mcg.contains_edge(&1, &2));
    //     assert_eq!(mcg.edge_weight(0, 1), Some(&(0, 1)));
    //     assert_eq!(mcg.edge_weight(1, 2), Some(&(1, 1)));

    //     assert!(mcg.validate());
    // }

    // #[test]
    // fn test_back_propogate_multi() {
    //     use super::MonteCarloGraph;
    //     let mut mcg: MonteCarloGraph<u32> = MonteCarloGraph::new();
    //     let mut path = vec![0, 1];
    //     mcg.back_propogate(path.clone(), false);
    //     path.push(2);
    //     mcg.back_propogate(path.clone(), false);
    //     path.push(3);
    //     mcg.back_propogate(path.clone(), true);

    //     assert_eq!(mcg.edge_weight(0, 1), Some(&(1, 1)));
    //     assert_eq!(mcg.edge_weight(1, 2), Some(&(0, 1)));
    //     assert_eq!(mcg.edge_weight(2, 3), Some(&(1, 1)));

    //     let mut path = vec![0, 1];
    //     mcg.back_propogate(path.clone(), false);
    //     path.push(4);
    //     mcg.back_propogate(path.clone(), false);
    //     mcg.back_propogate(path.clone(), false);
    //     mcg.back_propogate(path.clone(), false);
    //     path.push(5);
    //     mcg.back_propogate(path.clone(), false);
    //     path.push(6);
    //     mcg.back_propogate(path.clone(), true);

    //     assert_eq!(mcg.edge_weight(0, 1), Some(&(1, 2)));
    //     assert_eq!(mcg.edge_weight(1, 2), Some(&(0, 1)));
    //     assert_eq!(mcg.edge_weight(2, 3), Some(&(1, 1)));
    //     assert_eq!(mcg.edge_weight(1, 4), Some(&(1, 1)));
    //     assert_eq!(mcg.edge_weight(4, 5), Some(&(0, 1)));
    //     assert_eq!(mcg.edge_weight(5, 6), Some(&(1, 1)));

    //     let mut path = vec![0, 9];
    //     mcg.back_propogate(path.clone(), false);
    //     path.push(4);
    //     mcg.back_propogate(path.clone(), false);

    //     assert_eq!(mcg.edge_weight(0, 9), Some(&(0, 1)));
    //     assert_eq!(mcg.edge_weight(9, 4), Some(&(1, 1)));

    //     path.push(11);
    //     mcg.back_propogate(path.clone(), false);
    //     path.push(12);
    //     mcg.back_propogate(path.clone(), true);

    //     assert_eq!(mcg.edge_weight(0, 1), Some(&(1, 3)));
    //     assert_eq!(mcg.edge_weight(1, 2), Some(&(0, 1)));
    //     assert_eq!(mcg.edge_weight(2, 3), Some(&(1, 1)));
    //     assert_eq!(mcg.edge_weight(1, 4), Some(&(2, 2)));
    //     assert_eq!(mcg.edge_weight(4, 5), Some(&(0, 1)));
    //     assert_eq!(mcg.edge_weight(5, 6), Some(&(1, 1)));
    //     assert_eq!(mcg.edge_weight(0, 9), Some(&(0, 2)));
    //     assert_eq!(mcg.edge_weight(9, 4), Some(&(2, 2)));
    //     assert_eq!(mcg.edge_weight(4, 11), Some(&(0, 1)));
    //     assert_eq!(mcg.edge_weight(11, 12), Some(&(1, 1)));

    //     assert!(mcg.validate());
    // }

    // #[test]
    // fn test_back_propogate_path() {
    //     use super::MonteCarloGraph;
    //     let mut mcg: MonteCarloGraph<u32> = MonteCarloGraph::new();
    //     let path = vec![0, 1, 2, 3];
    //     mcg.back_propogate_path(path.clone(), true);

    //     assert!(mcg.contains_node(&0));
    //     assert!(mcg.contains_node(&1));
    //     assert!(mcg.contains_node(&2));
    //     assert!(mcg.contains_node(&3));
    //     assert!(mcg.contains_edge(&0, &1));
    //     assert!(mcg.contains_edge(&1, &2));
    //     assert!(mcg.contains_edge(&2, &3));
    //     assert_eq!(mcg.edge_weight(0, 1), Some(&(1, 1)));
    //     assert_eq!(mcg.edge_weight(1, 2), Some(&(0, 1)));
    //     assert_eq!(mcg.edge_weight(2, 3), Some(&(1, 1)));

    //     assert!(mcg.validate());
    // }

    #[test]
    fn test_back_propogate() {
        use super::MonteCarloGraph;
        use crate::BoardStatus;
        let mut mcg: MonteCarloGraph<u32> = MonteCarloGraph::new();
        let path = vec![0, 1, 2, 3];
        mcg.back_propogate(path.clone(), BoardStatus::Win(0));

        assert!(mcg.contains_node(&0));
        assert!(mcg.contains_node(&1));
        assert!(mcg.contains_node(&2));
        assert!(mcg.contains_node(&3));
        assert!(mcg.contains_edge(&0, &1));
        assert!(mcg.contains_edge(&1, &2));
        assert!(mcg.contains_edge(&2, &3));
        assert_eq!(mcg.edge_weight(0, 1), Some(&(1, 1)));
        assert_eq!(mcg.edge_weight(1, 2), Some(&(0, 1)));
        assert_eq!(mcg.edge_weight(2, 3), Some(&(1, 1)));

        assert!(mcg.validate());
    }

    #[test]
    fn test_back_propogate_multi() {
        use super::MonteCarloGraph;
        use crate::BoardStatus;
        let mut mcg: MonteCarloGraph<u32> = MonteCarloGraph::new();
        let path1 = vec![0, 1, 2, 3];
        mcg.back_propogate(path1.clone(), BoardStatus::Win(0));
        let path2 = vec![0, 1, 4, 5, 6];
        mcg.back_propogate(path2.clone(), BoardStatus::Win(0));
        let path3 = vec![0, 9, 4, 11, 12];
        mcg.back_propogate(path3.clone(), BoardStatus::Win(0));

        assert_eq!(mcg.edge_weight(0, 1), Some(&(1, 3)));
        assert_eq!(mcg.edge_weight(1, 2), Some(&(0, 1)));
        assert_eq!(mcg.edge_weight(2, 3), Some(&(1, 1)));
        assert_eq!(mcg.edge_weight(1, 4), Some(&(2, 2)));
        assert_eq!(mcg.edge_weight(4, 5), Some(&(0, 1)));
        assert_eq!(mcg.edge_weight(5, 6), Some(&(1, 1)));
        assert_eq!(mcg.edge_weight(0, 9), Some(&(0, 2)));
        assert_eq!(mcg.edge_weight(9, 4), Some(&(2, 2)));
        assert_eq!(mcg.edge_weight(4, 11), Some(&(0, 1)));
        assert_eq!(mcg.edge_weight(11, 12), Some(&(1, 1)));

        assert!(mcg.validate());
    }
}
