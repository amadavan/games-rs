use std::hash::RandomState;

use petgraph::algo;
use petgraph::visit::EdgeRef;
use petgraph::{Direction::Outgoing, prelude::DiGraphMap};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub struct MonteCarloGraph<N>
where
    N: std::hash::Hash
        + Eq
        + Clone
        + Copy
        + Ord
        + Default
        + std::fmt::Debug
        + Serialize
        + for<'de> Deserialize<'de>,
{
    // Graph structure and methods would be defined here
    graph: DiGraphMap<N, (usize, usize)>,
    root: N,
}

impl<N> Serialize for MonteCarloGraph<N>
where
    N: std::hash::Hash
        + Eq
        + Clone
        + Copy
        + Ord
        + Default
        + std::fmt::Debug
        + Serialize
        + for<'de> Deserialize<'de>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("MonteCarloGraph", 2)?;
        state.serialize_field("graph", &self.graph)?;
        state.serialize_field("root", &self.root)?;
        state.end()
    }
}

impl<'de, N> Deserialize<'de> for MonteCarloGraph<N>
where
    N: std::hash::Hash
        + Eq
        + Clone
        + Copy
        + Ord
        + Default
        + std::fmt::Debug
        + Serialize
        + for<'de2> Deserialize<'de2>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct MonteCarloGraphVisitor<N>(std::marker::PhantomData<N>);

        impl<'de, N> Visitor<'de> for MonteCarloGraphVisitor<N>
        where
            N: std::hash::Hash
                + Eq
                + Clone
                + Copy
                + Ord
                + Default
                + std::fmt::Debug
                + Serialize
                + for<'de2> Deserialize<'de2>,
        {
            type Value = MonteCarloGraph<N>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct MonteCarloGraph")
            }

            fn visit_map<V>(self, mut map: V) -> Result<MonteCarloGraph<N>, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut graph = None;
                let mut root = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "graph" => {
                            if graph.is_some() {
                                return Err(de::Error::duplicate_field("graph"));
                            }
                            graph = Some(map.next_value()?);
                        }
                        "root" => {
                            if root.is_some() {
                                return Err(de::Error::duplicate_field("root"));
                            }
                            root = Some(map.next_value()?);
                        }
                        _ => {
                            let _ = map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }
                let graph = graph.ok_or_else(|| de::Error::missing_field("graph"))?;
                let root = root.ok_or_else(|| de::Error::missing_field("root"))?;
                Ok(MonteCarloGraph { graph, root })
            }
        }

        const FIELDS: &[&str] = &["graph", "root"];
        deserializer.deserialize_struct(
            "MonteCarloGraph",
            FIELDS,
            MonteCarloGraphVisitor(std::marker::PhantomData),
        )
    }
}

impl<N> MonteCarloGraph<N>
where
    N: std::hash::Hash
        + Eq
        + Clone
        + Copy
        + Ord
        + Default
        + std::fmt::Debug
        + Serialize
        + for<'de> Deserialize<'de>,
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

    pub fn edge_weight(&self, from: N, to: N) -> Option<&(usize, usize)> {
        self.graph.edge_weight(from, to)
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

        // TODO: need to consider draws
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

    pub fn to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let serialized = bincode::serde::encode_to_vec(&self, bincode::config::standard())?;
        std::fs::write(path, serialized)?;
        Ok(())
    }

    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let data = std::fs::read(path)?;
        let (deserialized, _): (Self, _) =
            bincode::serde::decode_from_slice(&data, bincode::config::standard())?;
        Ok(deserialized)
    }
}
