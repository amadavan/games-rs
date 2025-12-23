//! Monte Carlo Graph Search data structure for game tree exploration.
//!
//! This module implements a graph-based approach to Monte Carlo tree search,
//! where game states are nodes and transitions are edges weighted with
//! win/simulation statistics.

use std::{
    collections::HashSet,
    ops::{Add, AddAssign},
};

use petgraph::Direction;
use petgraph::prelude::DiGraphMap;
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};

use crate::BoardStatus;

use derive_aliases::derive;

/// Game state classification.
///
/// # Examples
/// ```
/// use games_rs::agents::monte_carlo_graph::State;
///
/// let state = State::InProgress;
/// assert_eq!(state, State::InProgress);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// A player has won the game
    Win,
    /// The game ended in a draw
    Draw,
    /// The game is still ongoing
    InProgress,
}

/// Statistics for a game transition edge.
///
/// Tracks outcomes from the parent node's perspective.
///
/// # Examples
/// ```
/// use games_rs::agents::monte_carlo_graph::EdgeWeight;
///
/// let weight: EdgeWeight = (5, 3, 2).into();
/// assert_eq!(weight.wins(), 5);
/// assert_eq!(weight.losses(), 3);
/// assert_eq!(weight.draws(), 2);
/// assert_eq!(weight.simulations(), 10);
/// ```
#[derive(..StdTraits, Serialize, Deserialize, Debug)]
pub struct EdgeWeight {
    wins: usize,
    losses: usize,
    draws: usize,
}

impl EdgeWeight {
    /// Swaps wins and losses (for opponent's perspective).
    ///
    /// # Examples
    /// ```
    /// use games_rs::agents::monte_carlo_graph::EdgeWeight;
    ///
    /// let weight: EdgeWeight = (5, 3, 2).into();
    /// let flipped = weight.flip();
    /// assert_eq!(flipped.wins(), 3);
    /// assert_eq!(flipped.losses(), 5);
    /// ```
    pub fn flip(&self) -> EdgeWeight {
        EdgeWeight {
            wins: self.losses,
            losses: self.wins,
            draws: self.draws,
        }
    }

    /// Returns the number of wins.
    pub fn wins(&self) -> usize {
        self.wins
    }

    /// Returns the number of losses.
    pub fn losses(&self) -> usize {
        self.losses
    }

    /// Returns the number of draws.
    pub fn draws(&self) -> usize {
        self.draws
    }

    /// Returns total simulations (wins + losses + draws).
    pub fn simulations(&self) -> usize {
        self.wins + self.losses + self.draws
    }
}

impl Add<EdgeWeight> for EdgeWeight {
    type Output = EdgeWeight;

    fn add(self, rhs: EdgeWeight) -> Self::Output {
        EdgeWeight {
            wins: self.wins + rhs.wins,
            losses: self.losses + rhs.losses,
            draws: self.draws + rhs.draws,
        }
    }
}

impl AddAssign<EdgeWeight> for EdgeWeight {
    fn add_assign(&mut self, rhs: EdgeWeight) {
        self.wins += rhs.wins;
        self.losses += rhs.losses;
        self.draws += rhs.draws;
    }
}

impl AddAssign<EdgeWeight> for &mut EdgeWeight {
    fn add_assign(&mut self, rhs: EdgeWeight) {
        self.wins += rhs.wins;
        self.losses += rhs.losses;
        self.draws += rhs.draws;
    }
}

impl From<(usize, usize, usize)> for EdgeWeight {
    fn from(value: (usize, usize, usize)) -> Self {
        EdgeWeight {
            wins: value.0,
            losses: value.1,
            draws: value.2,
        }
    }
}

impl From<EdgeWeight> for (usize, usize, usize) {
    fn from(value: EdgeWeight) -> Self {
        (value.wins, value.losses, value.draws)
    }
}

/// Monte Carlo tree/graph search structure for game state exploration.
///
/// Tracks game states (nodes) and transitions (edges) with win/simulation statistics.
/// Edge weights represent (wins, losses, draws) from the parent node's perspective.
///
/// # Examples
/// ```
/// use games_rs::agents::monte_carlo_graph::MonteCarloGraph;
/// use games_rs::BoardStatus;
///
/// let mut graph: MonteCarloGraph<u32> = MonteCarloGraph::new();
/// let path = vec![0, 1, 2];
/// graph.back_propogate(path, BoardStatus::Win(0));
///
/// assert!(graph.contains_node(&1));
/// assert!(graph.contains_edge(&0, &1));
/// ```
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
    graph: DiGraphMap<N, EdgeWeight>,
    /// Root node representing the initial game state
    root: N,
}

impl<N> MonteCarloGraph<N>
where
    N: std::hash::Hash + Eq + Clone + Copy + Ord + Default + std::fmt::Debug + Serialize,
    for<'a> N: Deserialize<'a>,
{
    /// Creates a new graph with the default node as root.
    ///
    /// # Examples
    /// ```
    /// use games_rs::agents::monte_carlo_graph::MonteCarloGraph;
    ///
    /// let graph: MonteCarloGraph<u32> = MonteCarloGraph::new();
    /// assert!(graph.contains_node(&0));
    /// ```
    pub fn new() -> Self {
        let mut graph = DiGraphMap::new();
        graph.add_node(N::default());
        MonteCarloGraph {
            graph,
            root: N::default(),
        }
    }

    /// Aggregates outcomes from all outgoing edges.
    ///
    /// # Examples
    /// ```
    /// use games_rs::agents::monte_carlo_graph::MonteCarloGraph;
    /// use games_rs::BoardStatus;
    ///
    /// let mut graph: MonteCarloGraph<u32> = MonteCarloGraph::new();
    /// graph.back_propogate(vec![0, 1], BoardStatus::Win(0));
    /// graph.back_propogate(vec![0, 2], BoardStatus::Draw);
    ///
    /// let outcomes = graph.get_aggregate_outcomes(&0);
    /// assert_eq!(outcomes.simulations(), 2);
    /// ```
    #[inline]
    pub fn get_aggregate_outcomes(&self, node: &N) -> EdgeWeight {
        self.edges_from(node)
            .iter()
            .fold((0usize, 0usize, 0usize).into(), |weight, edge| {
                weight + edge.1
            })
    }

    /// Returns all nodes in the graph.
    #[inline]
    pub fn nodes(&self) -> Vec<N> {
        self.graph.nodes().collect()
    }

    /// Checks if a node exists in the graph.
    #[inline]
    pub fn contains_node(&self, n: &N) -> bool {
        self.graph.contains_node(*n)
    }

    /// Returns all incoming edges with their weights.
    #[inline]
    pub fn edges_to(&self, n: &N) -> Vec<(N, EdgeWeight)> {
        self.graph
            .edges_directed(*n, Direction::Incoming)
            .map(|e| (e.source(), *e.weight()))
            .collect()
    }

    /// Returns all outgoing edges with their weights.
    #[inline]
    pub fn edges_from(&self, n: &N) -> Vec<(N, EdgeWeight)> {
        self.graph
            .edges_directed(*n, Direction::Outgoing)
            .map(|e| (e.target(), *e.weight()))
            .collect()
    }

    /// Checks if an edge exists between two nodes.
    #[inline]
    pub fn contains_edge(&self, from: &N, to: &N) -> bool {
        self.graph.contains_edge(*from, *to)
    }

    /// Returns the weight of an edge, or None if it doesn't exist.
    ///
    /// # Examples
    /// ```
    /// use games_rs::agents::monte_carlo_graph::MonteCarloGraph;
    /// use games_rs::BoardStatus;
    ///
    /// let mut graph: MonteCarloGraph<u32> = MonteCarloGraph::new();
    /// graph.back_propogate(vec![0, 1], BoardStatus::Win(0));
    ///
    /// let weight = graph.edge_weight(0, 1).unwrap();
    /// assert_eq!(weight.wins(), 1);
    /// assert_eq!(weight.losses(), 0);
    /// ```
    #[inline]
    pub fn edge_weight(&self, from: N, to: N) -> Option<&EdgeWeight> {
        self.graph.edge_weight(from, to)
    }

    /// Propagates weight updates up the tree recursively.
    fn propogate_edge(&mut self, n: &N, weight_update: EdgeWeight) {
        for (src, _) in self.edges_to(n) {
            if let Some(weight) = self.graph.edge_weight_mut(src, *n) {
                *weight += weight_update.flip();
                self.propogate_edge(&src, weight_update.flip());
            }
        }
    }

    /// Updates the graph with simulation results from a game path.
    ///
    /// Creates nodes/edges as needed and propagates outcome statistics upward.
    ///
    /// # Examples
    /// ```
    /// use games_rs::agents::monte_carlo_graph::MonteCarloGraph;
    /// use games_rs::BoardStatus;
    ///
    /// let mut graph: MonteCarloGraph<u32> = MonteCarloGraph::new();
    /// graph.back_propogate(vec![0, 1, 2], BoardStatus::Win(0));
    ///
    /// assert!(graph.contains_edge(&0, &1));
    /// assert!(graph.contains_edge(&1, &2));
    /// assert!(graph.validate());
    /// ```
    pub fn back_propogate(&mut self, path: Vec<N>, state: BoardStatus) {
        for i in (1..path.len()) {
            let from = path[i - 1];
            let to = path[i];

            if !self.graph.contains_node(to) {
                self.graph.add_node(to);
            }

            if !self.contains_edge(&from, &to) {
                let weight = self.get_aggregate_outcomes(&to);
                self.graph.add_edge(from, to, weight.flip());
                self.propogate_edge(&from, weight.flip());
            }
        }

        if state != BoardStatus::InProgress || self.contains_node(&path[path.len() - 1]) {
            let weight: EdgeWeight = match state {
                BoardStatus::Win(_) => (1, 0, 0).into(),
                BoardStatus::Draw => (0, 0, 1).into(),
                _ => panic!("Invalid board status"),
            };

            self.propogate_edge(&path[path.len() - 1], weight.flip());
        }
    }

    /// Validates graph integrity.
    ///
    /// Checks that each non-leaf node's incoming edge weights match its aggregated outgoing edges.
    ///
    /// # Examples
    /// ```
    /// use games_rs::agents::monte_carlo_graph::MonteCarloGraph;
    /// use games_rs::BoardStatus;
    ///
    /// let mut graph: MonteCarloGraph<u32> = MonteCarloGraph::new();
    /// graph.back_propogate(vec![0, 1, 2], BoardStatus::Win(0));
    /// assert!(graph.validate());
    /// ```
    pub fn validate(&self) -> bool {
        let mut valid = true;
        self.graph
            .nodes()
            .filter(|n| self.edges_from(n).iter().count() > 0)
            .for_each(|n| {
                let exp_weight = self.get_aggregate_outcomes(&n).flip();
                valid &= self
                    .edges_to(&n)
                    .iter()
                    .all(|(_, weight)| *weight == exp_weight)
            });
        valid
    }

    /// Serializes the graph to a file using bitcode.
    ///
    /// # Examples
    /// ```no_run
    /// use games_rs::agents::monte_carlo_graph::MonteCarloGraph;
    ///
    /// let graph: MonteCarloGraph<u32> = MonteCarloGraph::new();
    /// graph.to_file("graph.bin").unwrap();
    /// ```
    pub fn to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let serialized = bitcode::serialize(self)?;
        std::fs::write(path, serialized)?;
        Ok(())
    }

    /// Deserializes the graph from a bitcode file.
    ///
    /// # Examples
    /// ```no_run
    /// use games_rs::agents::monte_carlo_graph::MonteCarloGraph;
    ///
    /// let graph: MonteCarloGraph<u32> = MonteCarloGraph::from_file("graph.bin").unwrap();
    /// ```
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let data = std::fs::read(path)?;
        let deserialized: Self = bitcode::deserialize(&data)?;
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
        mcg.graph.add_edge(0, 1, (1, 4, 0).into());
        mcg.graph.add_edge(1, 2, (3, 2, 1).into());

        assert!(mcg.contains_node(&0));
        assert!(mcg.contains_node(&1));
        assert!(mcg.contains_node(&2));
        assert!(mcg.contains_node(&5));
        assert!(mcg.contains_edge(&0, &1));
        assert!(mcg.contains_edge(&1, &2));
        assert_eq!(mcg.edge_weight(0, 1), Some(&(1, 4, 0).into()));
        assert_eq!(mcg.edge_weight(1, 2), Some(&(3, 2, 1).into()));

        let path = "test_mcg.bin";
        mcg.to_file(path).unwrap();

        let loaded_mcg = MonteCarloGraph::from_file(path).unwrap();

        assert!(loaded_mcg.contains_node(&0));
        assert!(loaded_mcg.contains_node(&1));
        assert!(loaded_mcg.contains_node(&2));
        assert!(loaded_mcg.contains_edge(&0, &1));
        assert!(loaded_mcg.contains_edge(&1, &2));
        assert_eq!(loaded_mcg.edge_weight(0, 1), Some(&(1, 4, 0).into()));
        assert_eq!(loaded_mcg.edge_weight(1, 2), Some(&(3, 2, 1).into()));

        std::fs::remove_file(path).unwrap();
    }

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
        assert_eq!(mcg.edge_weight(0, 1), Some(&(1, 0, 0).into()));
        assert_eq!(mcg.edge_weight(1, 2), Some(&(0, 1, 0).into()));
        assert_eq!(mcg.edge_weight(2, 3), Some(&(1, 0, 0).into()));

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

        assert_eq!(mcg.edge_weight(0, 1), Some(&(1, 2, 0).into()));
        assert_eq!(mcg.edge_weight(1, 2), Some(&(0, 1, 0).into()));
        assert_eq!(mcg.edge_weight(2, 3), Some(&(1, 0, 0).into()));
        assert_eq!(mcg.edge_weight(1, 4), Some(&(2, 0, 0).into()));
        assert_eq!(mcg.edge_weight(4, 5), Some(&(0, 1, 0).into()));
        assert_eq!(mcg.edge_weight(5, 6), Some(&(1, 0, 0).into()));
        assert_eq!(mcg.edge_weight(0, 9), Some(&(0, 2, 0).into()));
        assert_eq!(mcg.edge_weight(9, 4), Some(&(2, 0, 0).into()));
        assert_eq!(mcg.edge_weight(4, 11), Some(&(0, 1, 0).into()));
        assert_eq!(mcg.edge_weight(11, 12), Some(&(1, 0, 0).into()));

        assert!(mcg.validate());
    }
}
