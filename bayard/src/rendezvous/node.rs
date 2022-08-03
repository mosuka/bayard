use std::cmp;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};

use super::hash::NodeHasher;

/// This trait represents a candidate node for rendezvous.
pub trait Node {
    /// Node identifier type.
    type NodeId: Hash + PartialEq + Ord;

    /// Hash code type.
    type HashCode: Ord;

    /// Returns the identifier of this node.
    fn node_id(&self) -> &Self::NodeId;

    /// Returns the hash code for the combination of thid node and `item`.
    ///
    /// Note that the time complexity of this function should be constant.
    fn hash_code<H, U: Hash>(&self, hasher: &H, item: &U) -> Self::HashCode
    where
        H: NodeHasher<Self::NodeId>;
}

impl<T: Node> Node for &T {
    type NodeId = T::NodeId;
    type HashCode = T::HashCode;
    fn node_id(&self) -> &Self::NodeId {
        (**self).node_id()
    }
    fn hash_code<H, U: Hash>(&self, hasher: &H, item: &U) -> Self::HashCode
    where
        H: NodeHasher<Self::NodeId>,
    {
        (**self).hash_code(hasher, item)
    }
}

impl<'a> Node for &'a str {
    type NodeId = Self;
    type HashCode = u64;
    fn node_id(&self) -> &Self::NodeId {
        self
    }
    fn hash_code<H, U: Hash>(&self, hasher: &H, item: &U) -> Self::HashCode
    where
        H: NodeHasher<Self::NodeId>,
    {
        hasher.hash(self, item)
    }
}

/// Identity node.
#[derive(Debug, Clone)]
pub struct IdNode<T>(T);

impl<T> IdNode<T> {
    /// Makes a new `IdNode` instance.
    pub fn new(node: T) -> Self {
        IdNode(node)
    }

    /// Converts into inner node `T`.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for IdNode<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for IdNode<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Node for IdNode<T>
where
    T: Hash + PartialEq + Ord,
{
    type NodeId = T;
    type HashCode = u64;
    fn node_id(&self) -> &Self::NodeId {
        &self.0
    }
    fn hash_code<H, U: Hash>(&self, hasher: &H, item: &U) -> Self::HashCode
    where
        H: NodeHasher<Self::NodeId>,
    {
        hasher.hash(self.node_id(), item)
    }
}

/// Key-Value node.
#[derive(Debug, Clone)]
pub struct KeyValueNode<K, V> {
    /// The key of this node.
    pub key: K,

    /// The value of this node.
    pub value: V,
}

impl<K, V> KeyValueNode<K, V> {
    /// Makes a new `KeyValueNode` instance.
    ///
    /// This is equivalent to `KeyValueNode{node: node, value: value}`.
    pub fn new(key: K, value: V) -> Self {
        KeyValueNode { key, value }
    }
}

impl<K, V> Node for KeyValueNode<K, V>
where
    K: Hash + PartialEq + Ord,
{
    type NodeId = K;
    type HashCode = u64;
    fn node_id(&self) -> &Self::NodeId {
        &self.key
    }
    fn hash_code<H, U: Hash>(&self, hasher: &H, item: &U) -> Self::HashCode
    where
        H: NodeHasher<Self::NodeId>,
    {
        hasher.hash(self.node_id(), item)
    }
}

/// Wrapper of a `f64` value in which `f64::is_sign_positive(self.0)` is always true.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct SignPositiveF64(f64);

impl Eq for SignPositiveF64 {}

// TODO: Fix derive_ord_xor_partial_ord
#[allow(clippy::derive_ord_xor_partial_ord)]
impl cmp::Ord for SignPositiveF64 {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}

/// The capacity of a weighted node.
///
/// "capacity" is a virtual value indicating the resource amount of a node.
/// For example, if the capacity of a node is twice the other,
/// the former may be selected by items twice as many times as the latter.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Capacity(f64);

impl Capacity {
    /// Makes a new `Capacity` instance.
    ///
    /// Note that `capacity` must be a normal and positive value.
    /// If a value which breaks the condition
    /// `value.is_normal() && value.is_sign_positive()` is passed,
    /// this function willl return `None`.
    pub fn new(value: f64) -> Option<Self> {
        if value.is_normal() && value.is_sign_positive() {
            Some(Capacity(value))
        } else {
            None
        }
    }

    /// Returns the value of this instance.
    pub fn value(self) -> f64 {
        self.0
    }
}

/// Weighted node.
///
/// This is used for representing a heterogeneous environment in which
/// there are nodes which have various capacities.
///
/// Internally this uses an efficient weighted hash function that
/// based on the "Logarithmic Method" described in the paper "Weighted Distributed Hash Tables".
/// So, normally, additional cost for considering node capacity is negligible.
#[derive(Debug, Clone)]
pub struct WeightedNode<N> {
    /// The node.
    pub node: N,

    /// The capacity of this node.
    pub capacity: Capacity,
}

impl<N: Node> WeightedNode<N> {
    /// Makes a new `WeightedNode` instance.
    pub fn new(node: N, capacity: Capacity) -> Self {
        WeightedNode { node, capacity }
    }
}

impl<N: Node> Node for WeightedNode<N> {
    type NodeId = N::NodeId;
    type HashCode = SignPositiveF64;
    fn node_id(&self) -> &Self::NodeId {
        self.node.node_id()
    }
    fn hash_code<H, U: Hash>(&self, hasher: &H, item: &U) -> Self::HashCode
    where
        H: NodeHasher<Self::NodeId>,
    {
        use std::u64::MAX;
        let hash = hasher.hash(self.node_id(), item) as f64;
        let distance = (hash / MAX as f64).ln();
        SignPositiveF64(distance / self.capacity.0)
    }
}
