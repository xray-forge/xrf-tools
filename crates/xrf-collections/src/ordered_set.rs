use indexmap::IndexSet;

use crate::fast_build_hasher::FastBuildHasher;

/// A set that keeps its values in the order they were first inserted. Hashed with [`FastBuildHasher`]; build one with
/// `OrderedSet::default()`.
pub type OrderedSet<T> = IndexSet<T, FastBuildHasher>;
