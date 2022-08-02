//! An implementation of Rendezvous (a.k.a, highest random weight) hashing algorithm.
//!
//! # Examples
//!
//! For homogeneous nodes:
//!
//! ```
//! // Constructs a node (a.k.a., server, site, etc) set.
//! use parabuteo::rendezvous::hash::RendezvousNodes;
//!
//! let mut nodes = RendezvousNodes::default();
//! nodes.insert("foo");
//! nodes.insert("bar");
//! nodes.insert("baz");
//! nodes.insert("qux");
//!
//! // Finds candidate nodes for an item (a.k.a., object).
//! assert_eq!(nodes.calc_candidates(&1).collect::<Vec<_>>(),
//!            [&"bar", &"baz", &"foo", &"qux"]);
//! assert_eq!(nodes.calc_candidates(&"key").collect::<Vec<_>>(),
//!            [&"qux", &"bar", &"foo", &"baz"]);
//!
//! // Update the node set.
//! // (The relative order between existing nodes are preserved)
//! nodes.remove(&"baz");
//! assert_eq!(nodes.calc_candidates(&1).collect::<Vec<_>>(),
//!            [&"bar", &"foo", &"qux"]);
//! assert_eq!(nodes.calc_candidates(&"key").collect::<Vec<_>>(),
//!            [&"qux", &"bar", &"foo"]);
//! ```
//!
//! For heterogeneous nodes:
//!
//! ```
//! use std::collections::HashMap;
//! use parabuteo::rendezvous::hash::RendezvousNodes;
//! use parabuteo::rendezvous::node::{WeightedNode, Capacity};
//!
//! let mut nodes = RendezvousNodes::default();
//! nodes.insert(WeightedNode::new("foo", Capacity::new(70.0).unwrap()));
//! nodes.insert(WeightedNode::new("bar", Capacity::new(20.0).unwrap()));
//! nodes.insert(WeightedNode::new("baz", Capacity::new(9.0).unwrap()));
//! nodes.insert(WeightedNode::new("qux", Capacity::new(1.0).unwrap()));
//!
//! let mut counts = HashMap::new();
//! for item in 0..10000 {
//!     let node = nodes.calc_candidates(&item).nth(0).unwrap();
//!     *counts.entry(node.node.to_string()).or_insert(0) += 1;
//! }
//! assert_eq!(((counts["foo"] as f64) / 100.0).round(), 70.0);
//! assert_eq!(((counts["bar"] as f64) / 100.0).round(), 20.0);
//! assert_eq!(((counts["baz"] as f64) / 100.0).round(), 9.0);
//! assert_eq!(((counts["qux"] as f64) / 100.0).round(), 1.0);
//! ```
#![warn(missing_docs)]
use std::borrow::Borrow;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub use super::node::{Capacity, IdNode, KeyValueNode, Node, WeightedNode};

pub mod types {
    //! Miscellaneous types.
    pub use crate::rendezvous::node::SignPositiveF64;
}

/// This trait allows calculating the hash value of a node for a specific item.
pub trait NodeHasher<N> {
    /// Returns the hash value for the combination of `node` and `item`.
    fn hash<T: Hash>(&self, node: &N, item: &T) -> u64;
}

/// The default `NodeHasher` implementation.
///
/// This uses `DefaultHasher` to hash nodes and items.
/// `DefaultHasher` is provided by Rust standard library.
///
/// To hash a combination of a node and an item,
/// `DefaultNodeHasher` hashes the item at first,
/// then hashes the node,
/// and finally returns the resulting hash value
/// (as follows).
///
/// ```no_run
/// use std::collections::hash_map::DefaultHasher;
/// # use std::hash::{Hash, Hasher};
/// # let item = ();
/// # let node = ();
///
/// let mut hasher = DefaultHasher::new();
/// item.hash(&mut hasher);
/// node.hash(&mut hasher);
/// hasher.finish()
/// # ;
/// ```
#[derive(Debug, Default, Clone)]
pub struct DefaultNodeHasher(());

impl DefaultNodeHasher {
    /// Makes a new `DefaultNodeHasher` instance.
    pub fn new() -> Self {
        DefaultNodeHasher(())
    }
}

impl<N: Hash> NodeHasher<N> for DefaultNodeHasher {
    fn hash<T: Hash>(&self, node: &N, item: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        node.hash(&mut hasher);
        hasher.finish()
    }
}

/// A candidate node set of a rendezvous for clients that are requiring the same item.
#[derive(Clone, Debug)]
pub struct RendezvousNodes<N: Node, H> {
    nodes: Vec<N>,
    hasher: H,
    index: Arc<AtomicUsize>,
}

impl<N, H> RendezvousNodes<N, H>
where
    N: Node,
    H: NodeHasher<N::NodeId>,
{
    /// Makes a new `RendezvousNodes` instance.
    pub fn new(hasher: H) -> Self {
        RendezvousNodes {
            nodes: Vec::new(),
            hasher,
            index: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Returns the candidate nodes for `item`.
    ///
    /// The higher priority node is located in front of the returned candidate sequence.
    ///
    /// Note that this method takes `O(n log n)` steps
    /// (where `n` is the return value of `self.len()`).
    pub fn calc_candidates<T: Hash>(&self, item: &T) -> impl Iterator<Item = &N> {
        let hasher = &self.hasher;
        let mut nodes = Vec::with_capacity(self.nodes.len());
        for node in &self.nodes {
            let code = node.hash_code(hasher, &item);
            nodes.push((node, code));
        }
        nodes.sort_unstable_by(|a, b| (&b.1, b.0.node_id()).cmp(&(&a.1, a.0.node_id())));
        nodes.into_iter().map(|n| n.0)
    }

    /// Returns the top N candidate nodes for `item`.
    pub fn calc_top_n_candidates<T: Hash>(&self, item: &T, num: usize) -> impl Iterator<Item = &N> {
        self.calc_candidates(item).take(num)
    }

    /// Returns the rotated candidate node from the top N candidate nodes for `item`.
    pub fn rotate_candidate<T: Hash>(&self, item: &T, num: usize) -> Option<&N> {
        // Get candidates.
        let candidates = self.calc_top_n_candidates(item, num).collect::<Vec<_>>();

        // Get current index number.
        let index = self.index.load(Ordering::Relaxed);

        // Update index number.
        self.index
            .store((index + 1) % candidates.len(), Ordering::Relaxed);

        candidates.get(index).copied()
    }
}

impl<N: Node, H> RendezvousNodes<N, H> {
    /// Inserts a new candidate node.
    ///
    /// If a node which has an identifier equal to `node` exists,
    /// it will be removed and returned as `Some(N)`.
    pub fn insert(&mut self, node: N) -> Option<N> {
        let old = self.remove(node.node_id());
        self.nodes.push(node);
        old
    }

    /// Removes the specified node from the candidates.
    ///
    /// If the node does not exist, this method will return `None`.
    pub fn remove<M>(&mut self, node_id: &M) -> Option<N>
    where
        N::NodeId: Borrow<M>,
        M: PartialEq,
    {
        if let Some(i) = self
            .nodes
            .iter()
            .position(|n| n.node_id().borrow() == node_id)
        {
            Some(self.nodes.swap_remove(i))
        } else {
            None
        }
    }

    /// Returns `true` if the specified node exists in this candidate set, otherwise `false`.
    pub fn contains<M>(&self, node_id: &M) -> bool
    where
        N::NodeId: Borrow<M>,
        M: PartialEq,
    {
        self.nodes.iter().any(|n| n.node_id().borrow() == node_id)
    }

    /// Returns `true` if there are no candidate nodes.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns the count of the candidate nodes.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns an iterator over the nodes of this candidate set.
    pub fn iter(&self) -> impl Iterator<Item = &N> {
        self.nodes.iter()
    }

    /// Clears the candidate nodes.
    pub fn clear(&mut self) {
        self.nodes.clear();
    }
}

impl<N: Node> Default for RendezvousNodes<N, DefaultNodeHasher> {
    fn default() -> Self {
        Self::new(DefaultNodeHasher::new())
    }
}

impl<N: Node, H> IntoIterator for RendezvousNodes<N, H> {
    type Item = N;
    type IntoIter = std::vec::IntoIter<N>;
    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

impl<N: Node, H> Extend<N> for RendezvousNodes<N, H> {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = N>,
    {
        for n in iter {
            let _ = self.insert(n);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    macro_rules! assert_calc_candidates {
        ($nodes:expr, $key:expr, $candidates:expr) => {
            assert_eq!(
                $nodes.calc_candidates($key).collect::<Vec<_>>(),
                $candidates
            );
        };
    }

    macro_rules! assert_calc_top_n_candidates {
        ($nodes:expr, $key:expr, $num:expr, $candidates:expr) => {
            assert_eq!(
                $nodes.calc_top_n_candidates($key, $num).collect::<Vec<_>>(),
                $candidates
            );
        };
    }

    macro_rules! assert_rotate_candidate {
        ($nodes:expr, $key:expr, $num:expr, $candidate:expr) => {
            assert_eq!($nodes.rotate_candidate($key, $num), $candidate);
        };
    }

    #[test]
    fn it_works() {
        let mut nodes = RendezvousNodes::default();
        nodes.insert("foo");
        nodes.insert("bar");
        nodes.insert("baz");
        nodes.insert("qux");
        assert_calc_candidates!(nodes, &1, [&"bar", &"baz", &"foo", &"qux"]);
        assert_calc_candidates!(nodes, &"key", [&"qux", &"bar", &"foo", &"baz"]);

        nodes.remove(&"baz");
        assert_calc_candidates!(nodes, &1, [&"bar", &"foo", &"qux"]);
        assert_calc_candidates!(nodes, &"key", [&"qux", &"bar", &"foo"]);

        nodes.remove(&"bar");
        assert_calc_candidates!(nodes, &1, [&"foo", &"qux"]);
        assert_calc_candidates!(nodes, &"key", [&"qux", &"foo"]);

        nodes.insert("bar");
        nodes.insert("baz");
        let mut counts = HashMap::new();
        for item in 0..1000 {
            let node = nodes.calc_candidates(&item).nth(0).unwrap();
            *counts.entry(node.to_string()).or_insert(0) += 1;
        }
        assert_eq!(counts["foo"], 246);
        assert_eq!(counts["bar"], 266);
        assert_eq!(counts["baz"], 237);
        assert_eq!(counts["qux"], 251);
    }

    #[test]
    fn kv_nodes() {
        let mut nodes = RendezvousNodes::default();
        nodes.insert(KeyValueNode::new("foo", ()));
        nodes.insert(KeyValueNode::new("bar", ()));
        nodes.insert(KeyValueNode::new("baz", ()));
        nodes.insert(KeyValueNode::new("qux", ()));
        assert_eq!(
            nodes
                .calc_candidates(&1)
                .map(|n| &n.key)
                .collect::<Vec<_>>(),
            [&"bar", &"baz", &"foo", &"qux"]
        );
        assert_eq!(
            nodes
                .calc_candidates(&"key")
                .map(|n| &n.key)
                .collect::<Vec<_>>(),
            [&"qux", &"bar", &"foo", &"baz"]
        );
    }

    #[test]
    fn heterogeneous_nodes() {
        let mut nodes = RendezvousNodes::default();
        nodes.insert(WeightedNode::new("foo", Capacity::new(70.0).unwrap()));
        nodes.insert(WeightedNode::new("bar", Capacity::new(20.0).unwrap()));
        nodes.insert(WeightedNode::new("baz", Capacity::new(9.0).unwrap()));
        nodes.insert(WeightedNode::new("qux", Capacity::new(1.0).unwrap()));

        let mut counts = HashMap::new();
        for item in 0..10000 {
            let node = nodes.calc_candidates(&item).nth(0).unwrap();
            *counts.entry(node.node.to_string()).or_insert(0) += 1;
        }
        assert_eq!(((counts["foo"] as f64) / 100.0).round(), 70.0);
        assert_eq!(((counts["bar"] as f64) / 100.0).round(), 20.0);
        assert_eq!(((counts["baz"] as f64) / 100.0).round(), 9.0);
        assert_eq!(((counts["qux"] as f64) / 100.0).round(), 1.0);
    }

    #[test]
    fn calc_top_n_candidates() {
        let mut nodes = RendezvousNodes::default();
        nodes.insert("foo");
        nodes.insert("bar");
        nodes.insert("baz");
        nodes.insert("qux");
        assert_calc_top_n_candidates!(nodes, &1, 1, [&"bar"]);
        assert_calc_top_n_candidates!(nodes, &1, 2, [&"bar", &"baz"]);
        assert_calc_top_n_candidates!(nodes, &1, 3, [&"bar", &"baz", &"foo"]);
        assert_calc_top_n_candidates!(nodes, &1, 4, [&"bar", &"baz", &"foo", &"qux"]);
        assert_calc_top_n_candidates!(nodes, &1, 5, [&"bar", &"baz", &"foo", &"qux"]);
    }

    #[test]
    fn rotate_candidate() {
        let mut nodes = RendezvousNodes::default();
        nodes.insert("foo");
        nodes.insert("bar");
        nodes.insert("baz");
        nodes.insert("qux");
        assert_rotate_candidate!(nodes, &1, 3, Some(&"bar"));
        assert_rotate_candidate!(nodes, &1, 3, Some(&"baz"));
        assert_rotate_candidate!(nodes, &1, 3, Some(&"foo"));
        assert_rotate_candidate!(nodes, &1, 3, Some(&"bar"));
        assert_rotate_candidate!(nodes, &1, 3, Some(&"baz"));
        assert_rotate_candidate!(nodes, &1, 3, Some(&"foo"));
    }
}
