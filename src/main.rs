//! Graph example

#![forbid(missing_debug_implementations)]

use std::collections::hash_map::{Entry, HashMap};
use std::collections::{BTreeSet, VecDeque};
use std::fmt;

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
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
    edges: HashMap<Vertex, HashMap<Vertex, f32>>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            vertices: BTreeSet::new(),
            edges: HashMap::new(),
        }
    }

    pub fn add_rate(&mut self, edge: Edge, rate: f32) {
        assert!(rate != 0.0);
        if edge.0 == edge.1 {
            return;
        }
        self.vertices.insert(edge.0);
        self.vertices.insert(edge.1);
        let entry = self.edges.entry(edge.0).or_insert_with(HashMap::new);
        entry.insert(edge.1, rate);
        let entry = self.edges.entry(edge.1).or_insert_with(HashMap::new);
        entry.insert(edge.0, 1.0 / rate);
    }

    // Breath first traversal find a best rate.
    pub fn find_best_rate(&self, src: &Vertex, dst: &Vertex) -> Option<f32> {
        let mut visited = HashMap::new();
        let mut queue = VecDeque::new();
        let mut best_rate = None;

        queue.push_back((src, 1.0));
        while let Some((current_vertex, current_rate)) = queue.pop_front() {
            match visited.entry(current_vertex) {
                Entry::Vacant(entry) => {
                    entry.insert(current_rate);
                }
                Entry::Occupied(mut entry) => {
                    let current_entry = entry.get_mut();
                    if *current_entry > current_rate {
                        continue;
                    } else {
                        *current_entry = current_rate;
                    }
                }
            }
            if current_vertex == dst {
                best_rate = best_rate
                    .and_then(|best_rate| {
                        if current_rate > best_rate {
                            Some(current_rate)
                        } else {
                            Some(best_rate)
                        }
                    })
                    .or(Some(current_rate));
                continue;
            }
            match self.edges.get(current_vertex) {
                None => {
                    continue;
                }
                Some(vertices) => {
                    for (next_vertex, rate) in vertices {
                        let new_rate = rate * current_rate;
                        if visited.get(next_vertex).is_none() {
                            queue.push_back((next_vertex, new_rate));
                        }
                    }
                }
            }
        }

        best_rate
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
