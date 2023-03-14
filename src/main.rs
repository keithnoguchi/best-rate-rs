//! Dex example

#![forbid(missing_debug_implementations)]

use std::collections::hash_map::{Entry, HashMap};
use std::collections::{BTreeMap, VecDeque};
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

#[derive(Debug)]
struct Dex {
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
        let mut pushed = 0;

        queue.push_back((src, 1.0, vec![src]));
        while let Some((new_vertex, new_rate, path)) = queue.pop_front() {
            debug_assert!(pushed < 50);
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
                        trace!(%new_vertex, %current_rate, %new_rate, "new rate is better than current rate");
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
    use super::Dex;

    #[test]
    fn test_direct() {
        let mut dex = Dex::new();
        dex.add_rate('A', 'B', 1.4);
        dex.add_rate('A', 'C', 0.29);
        dex.add_rate('B', 'C', 0.2);

        let src = 'A'.into();
        let dst = 'C'.into();
        let rate = dex.find_best_rate(&src, &dst);
        assert_eq!(rate, Some(0.29));
    }

    #[test]
    fn test_one_hop() {
        let mut dex = Dex::new();
        dex.add_rate('A', 'B', 1.4);
        dex.add_rate('A', 'C', 0.1);
        dex.add_rate('B', 'C', 0.2);

        let src = 'A'.into();
        let dst = 'C'.into();
        let rate = dex.find_best_rate(&src, &dst);
        assert_eq!(rate, Some(0.28));
    }

    #[test]
    fn test_two_hops() {
        let mut dex = Dex::new();
        dex.add_rate('A', 'B', 1.4);
        dex.add_rate('A', 'C', 0.1);
        dex.add_rate('A', 'D', 0.055);
        dex.add_rate('B', 'C', 0.2);
        dex.add_rate('C', 'D', 0.2);
        dex.add_rate('D', 'F', 2.5);

        let src = 'A'.into();
        let dst = 'D'.into();
        let rate = dex.find_best_rate(&src, &dst);
        assert_eq!(rate, Some(0.056));
    }

    #[test]
    fn test_loop() {
        let mut dex = Dex::new();
        dex.add_rate('A', 'B', 1.4);
        dex.add_rate('A', 'C', 0.1);
        dex.add_rate('B', 'C', 0.2);
        dex.add_rate('C', 'D', 0.2);
        dex.add_rate('D', 'F', 2.5);

        let src = 'D'.into();
        let dst = 'F'.into();
        let rate = dex.find_best_rate(&src, &dst);
        assert_eq!(rate, Some(2.5));
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
