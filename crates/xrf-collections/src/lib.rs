#![doc = include_str!("../README.md")]

pub(crate) mod fast_build_hasher;
pub(crate) mod fast_map;
pub(crate) mod fast_set;
pub(crate) mod ordered_map;
pub(crate) mod ordered_set;

#[cfg(test)]
mod tests;

/// `hashbrown`'s raw table, for an index keyed by hashes it computes itself.
pub use hashbrown;
/// `indexmap`'s entry and iterator types, which [`OrderedMap`] and [`OrderedSet`] hand out.
pub use indexmap;
/// A lock-free ring of one writer and one reader, for handing values to a real-time thread without a lock.
pub use rtrb;

pub use crate::fast_build_hasher::FastBuildHasher;
pub use crate::fast_map::FastMap;
pub use crate::fast_set::FastSet;
pub use crate::ordered_map::OrderedMap;
pub use crate::ordered_set::OrderedSet;
