//! Dex example

#![forbid(missing_debug_implementations)]

use std::cmp::Ordering;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::{BTreeMap, VecDeque};
use std::fmt;

use tracing::{debug, instrument, trace};

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

impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        self.rate == other.rate
    }
}

impl PartialOrd for Path {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.rate.partial_cmp(&other.rate)
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, vertex) in self.path.iter().take(10).enumerate() {
            if i != 0 {
                let _ = f.write_fmt(format_args!(" -> "));
            }
            let _ = f.write_fmt(format_args!("{}", vertex));
        }
        f.write_fmt(format_args!(": {}", self.rate))
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

    pub fn last(&self) -> &Vertex {
        assert!(!self.is_empty());
        self.path.last().unwrap()
    }

    pub fn rate(&self) -> f32 {
        self.rate
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

#[derive(Debug, Default)]
pub struct Dex {
    edges: BTreeMap<Vertex, HashMap<Vertex, f32>>,
}

impl Dex {
    pub fn new() -> Self {
        Self::default()
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
    pub fn get_best_rate(&self, src: &Vertex, dst: &Vertex) -> Option<Path> {
        let mut visited = HashMap::new();
        let mut queue = VecDeque::new();
        let mut best_path = None;

        queue.push_back(Path::new(*src));
        while let Some(path) = queue.pop_front() {
            debug_assert!(path.len() < 100);
            trace!(%path, "queue.pop_front()");

            // The visited vertex check.
            //
            // It drops the vertex in case the newly calculated rate
            // is more than what we have in the visited HashMap.
            match visited.entry(*path.last()) {
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
                        trace!(%current_rate, %path, "new rate is better than current rate");
                        *current_rate = path.rate;
                    }
                }
            }

            // Update the rate in case the newly calculated rate
            // is better than what we have.
            if path.last() == dst {
                best_path = Some(match best_path {
                    Some(current_path) => {
                        if path > current_path {
                            debug!(%path, %current_path, "use the new path");
                            path
                        } else {
                            debug!(%path, %current_path, "use the current path");
                            current_path
                        }
                    }
                    None => path,
                });
            } else {
                // Continues the breath first search by pushing the new
                // vertex into to the `queue`.
                if let Some(vertices) = self.edges.get(path.last()) {
                    for (vertex, rate) in vertices {
                        if !path.contains(vertex) {
                            let mut path = path.clone();
                            path.insert(*vertex, *rate);
                            trace!(%path, "queue.push_back");
                            queue.push_back(path);
                        }
                    }
                }
            }
        }

        best_path
    }
}

#[cfg(test)]
mod test;

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
            if src != dst {
                if let Some(path) = dex.get_best_rate(src, dst) {
                    println!("{src} -> {dst}: {:8.4} ({path})", path.rate);
                }
            }
        }
    }
}
