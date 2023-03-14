//! Graph example

#![forbid(missing_debug_implementations)]

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
struct Vertex(char);

impl From<char> for Vertex {
    fn from(v: char) -> Self {
        Self(v)
    }
}

impl fmt::Display for Vertex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq)]
struct Edge(Vertex, Vertex);

impl From<[char; 2]> for Edge {
    fn from(mut edge: [char; 2]) -> Self {
        edge.sort();
        Self(edge[0].into(), edge[1].into())
    }
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self.0, self.1)
    }
}

#[derive(Debug)]
struct Graph {
    vertices: BTreeSet<Vertex>,
    edges: BTreeMap<Vertex, (Vertex, f32)>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            vertices: BTreeSet::new(),
            edges: BTreeMap::new(),
        }
    }

    pub fn add_rate(&mut self, edge: Edge, rate: f32) {
        self.vertices.insert(edge.0);
        self.vertices.insert(edge.1);
        self.edges.insert(edge.0, (edge.1, rate));
    }

    pub fn find_best_rate(&self, _src: &Vertex, _dst: &Vertex) -> Option<f32> {
        Some(1.0)
    }
}

fn main() {
    let mut graph = Graph::new();
    graph.add_rate(['A', 'B'].into(), 1.4);
    graph.add_rate(['B', 'C'].into(), 0.2);
    graph.add_rate(['D', 'F'].into(), 2.5);
    graph.add_rate(['A', 'C'].into(), 0.1);

    for src in &graph.vertices {
        for dst in &graph.vertices {
            if src >= dst {
                continue;
            }
            if let Some(rate) = graph.find_best_rate(src, dst) {
                println!("{src} -> {dst}: {rate}");
            }
        }
    }
}
