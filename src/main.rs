//! Dex example

#![forbid(missing_debug_implementations)]

use std::collections::hash_map::{Entry, HashMap};
use std::collections::{BTreeMap, VecDeque};
use std::fmt;

use tracing::{debug, instrument, trace};

#[cfg(test)]
mod test;

#[derive(Copy, Clone, Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct Vertex(char);

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

#[derive(Clone, Debug)]
pub struct Path {
    path: Vec<Vertex>,
    rate: f32,
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} -> {}", self.path, self.rate)
    }
}

impl Path {
    pub fn new(src: Vertex) -> Self {
        Self {
            path: vec![src],
            rate: 1.0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.path.is_empty()
    }

    pub fn len(&self) -> usize {
        self.path.len()
    }

    pub fn contains(&self, v: &Vertex) -> bool {
        self.path.contains(v)
    }

    pub fn insert(&mut self, v: Vertex, rate: f32) -> bool {
        if self.contains(&v) {
            return false;
        }
        self.path.push(v);
        self.rate *= rate;
        true
    }
}

#[derive(Debug)]
pub struct Dex {
    edges: BTreeMap<Vertex, HashMap<Vertex, f32>>,
}

impl Dex {
    pub fn new() -> Self {
        Self {
            edges: BTreeMap::new(),
        }
    }

    pub fn vertices(&self) -> impl Iterator<Item = &Vertex> {
        self.edges.keys()
    }

    pub fn add_rate(&mut self, src: char, dst: char, rate: f32) {
        assert!(src != dst && rate != 0.0);
        let src = src.into();
        let dst = dst.into();
        let entry = self.edges.entry(src).or_insert_with(HashMap::new);
        entry.insert(dst, rate);
        let entry = self.edges.entry(dst).or_insert_with(HashMap::new);
        entry.insert(src, 1.0 / rate);
    }

    // Breath first traversal to find the best rate.
    #[instrument(level = "debug", skip(self), ret)]
    pub fn find_best_rate(&self, src: &Vertex, dst: &Vertex) -> Option<f32> {
        let mut visited = HashMap::new();
        let mut queue = VecDeque::new();
        let mut best_rate = None;

        queue.push_back((src, Path::new(*src)));
        while let Some((new_vertex, path)) = queue.pop_front() {
            debug_assert!(path.len() < 100);
            trace!(%new_vertex, %path, "queue.pop_front()");

            // The visited vertex check.
            //
            // It drops the vertex in case the newly calculated rate
            // is more than what we have in the visited HashMap.
            match visited.entry(new_vertex) {
                Entry::Vacant(entry) => {
                    entry.insert(path.rate);
                }
                Entry::Occupied(mut entry) => {
                    let current_rate = entry.get_mut();
                    if *current_rate >= path.rate {
                        // Current one is better.  Skip this vertex.
                        continue;
                    } else {
                        // New one is better.  Continue the process.
                        trace!(%new_vertex, %current_rate, %path, "new rate is better than current rate");
                        *current_rate = path.rate;
                    }
                }
            }

            // Update the rate in case the newly calculated rate
            // is better than what we have.
            if new_vertex == dst {
                best_rate = best_rate
                    .map(|best_rate| {
                        if path.rate > best_rate {
                            debug!(%path.rate, %best_rate, "use the current rate");
                            path.rate
                        } else {
                            debug!(%path.rate, %best_rate, "use the new rate");
                            best_rate
                        }
                    })
                    .or(Some(path.rate));
                continue;
            }

            // Continues the breath first search by pushing the new
            // vertex into to the `queue`.
            if let Some(vertices) = self.edges.get(new_vertex) {
                for (vertex, rate) in vertices {
                    if !path.contains(vertex) {
                        let mut path = path.clone();
                        path.insert(*vertex, *rate);
                        trace!(%vertex, %path, "queue.push_back");
                        queue.push_back((vertex, path));
                    }
                }
            }
        }

        best_rate
    }
}

fn main() {
    let mut dex = Dex::new();
    dex.add_rate('A', 'B', 1.4);
    dex.add_rate('A', 'C', 0.1);
    dex.add_rate('B', 'C', 0.2);
    dex.add_rate('C', 'D', 0.2);
    dex.add_rate('D', 'F', 2.5);

    tracing_subscriber::fmt::init();
    trace!("{:#?}", dex);

    for src in dex.vertices() {
        for dst in dex.vertices() {
            if src >= dst {
                continue;
            }
            if let Some(rate) = dex.find_best_rate(src, dst) {
                println!("{src} -> {dst}: {rate}");
            }
        }
    }
}
