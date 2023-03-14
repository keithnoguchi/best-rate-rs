//! Graph example

#![forbid(missing_debug_implementations)]

use std::collections::hash_map::{Entry, HashMap};
use std::collections::{BTreeSet, VecDeque};
use std::fmt;

use tracing::{debug, instrument, trace};

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

    // Breath first traversal to find the best rate.
    #[instrument(level = "debug", skip(self), ret)]
    pub fn find_best_rate(&self, src: &Vertex, dst: &Vertex) -> Option<f32> {
        let mut visited = HashMap::new();
        let mut queue = VecDeque::new();
        let mut best_rate = None;
        let mut pushed = 0;

        queue.push_back((src, 1.0, vec![src]));
        while let Some((new_vertex, new_rate, path)) = queue.pop_front() {
            debug_assert!(pushed < 100);
            trace!(%new_vertex, %new_rate, ?path, "queue.pop_front()");

            // The visited vertex check.
            //
            // It drops the vertex in case the newly calculated rate
            // is more than what we have in the visited HashMap.
            match visited.entry(new_vertex) {
                Entry::Vacant(entry) => {
                    entry.insert(new_rate);
                }
                Entry::Occupied(mut entry) => {
                    let current_rate = entry.get_mut();
                    if *current_rate >= new_rate {
                        // Current one is better.  Skip this vertex.
                        continue;
                    } else {
                        // New one is better.  Continue the process.
                        let diff = new_rate - *current_rate;
                        trace!(%new_vertex, %current_rate, %new_rate, %diff, "new rate is better than current rate");
                        *current_rate = new_rate;
                    }
                }
            }

            // Update the rate in case the newly calculated rate
            // is better than what we have.
            if new_vertex == dst {
                best_rate = best_rate
                    .map(|best_rate| {
                        if new_rate > best_rate {
                            debug!(%new_rate, %best_rate, "use the current rate");
                            new_rate
                        } else {
                            debug!(%new_rate, %best_rate, "use the new rate");
                            best_rate
                        }
                    })
                    .or(Some(new_rate));
                continue;
            }

            // Continues the breath first search by pushing the new
            // vertex into to the `queue`.
            if let Some(vertices) = self.edges.get(new_vertex) {
                for (vertex, rate) in vertices {
                    if !path.contains(&vertex) {
                        let mut path = path.clone();
                        path.push(vertex);
                        let new_rate = rate * new_rate;
                        trace!(%vertex, %new_rate, "queue.push_back");
                        queue.push_back((vertex, new_rate, path));
                        pushed += 1;
                    }
                }
            }
        }

        best_rate
    }
}

#[cfg(test)]
mod test {
    use super::Graph;

    #[test]
    fn test_direct() {
        let mut graph = Graph::new();
        graph.add_rate(['A', 'B'].into(), 1.4);
        graph.add_rate(['A', 'C'].into(), 0.29);
        graph.add_rate(['B', 'C'].into(), 0.2);

        let src = 'A'.into();
        let dst = 'C'.into();
        let rate = graph.find_best_rate(&src, &dst);
        assert_eq!(rate, Some(0.29));
    }

    #[test]
    fn test_one_hop() {
        let mut graph = Graph::new();
        graph.add_rate(['A', 'B'].into(), 1.4);
        graph.add_rate(['A', 'C'].into(), 0.1);
        graph.add_rate(['B', 'C'].into(), 0.2);

        let src = 'A'.into();
        let dst = 'C'.into();
        let rate = graph.find_best_rate(&src, &dst);
        assert_eq!(rate, Some(0.28));
    }

    #[test]
    fn test_two_hops() {
        let mut graph = Graph::new();
        graph.add_rate(['A', 'B'].into(), 1.4);
        graph.add_rate(['A', 'C'].into(), 0.1);
        graph.add_rate(['A', 'D'].into(), 0.055);
        graph.add_rate(['B', 'C'].into(), 0.2);
        graph.add_rate(['C', 'D'].into(), 0.2);
        graph.add_rate(['D', 'F'].into(), 2.5);

        let src = 'A'.into();
        let dst = 'D'.into();
        let rate = graph.find_best_rate(&src, &dst);
        assert_eq!(rate, Some(0.056));
    }

    #[test]
    fn test_loop() {
        let mut graph = Graph::new();
        graph.add_rate(['A', 'B'].into(), 1.4);
        graph.add_rate(['A', 'C'].into(), 0.1);
        graph.add_rate(['B', 'C'].into(), 0.2);
        graph.add_rate(['C', 'D'].into(), 0.2);
        graph.add_rate(['D', 'F'].into(), 2.5);

        let src = 'D'.into();
        let dst = 'F'.into();
        let rate = graph.find_best_rate(&src, &dst);
        assert_eq!(rate, Some(2.5));
    }
}

fn main() {
    let mut graph = Graph::new();
    graph.add_rate(['A', 'B'].into(), 1.4);
    graph.add_rate(['A', 'C'].into(), 0.1);
    graph.add_rate(['B', 'C'].into(), 0.2);
    graph.add_rate(['C', 'D'].into(), 0.2);
    graph.add_rate(['D', 'F'].into(), 2.5);

    tracing_subscriber::fmt::init();
    trace!("{:#?}", graph);

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
