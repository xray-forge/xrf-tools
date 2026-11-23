use indexmap::IndexMap;

use crate::fast_build_hasher::FastBuildHasher;

/// A map that keeps its keys in the order they were first inserted, as a game file lists its sections and fields, so
/// what is read can be written back in the same order. Hashed with [`FastBuildHasher`].
///
/// Build one with `OrderedMap::default()` or `OrderedMap::with_capacity_and_hasher(n, FastBuildHasher)`; `new` and
/// `with_capacity` belong to the standard library's own hasher. The entry and iterator types are
/// `xrf_collections::indexmap::map`'s.
pub type OrderedMap<K, V> = IndexMap<K, V, FastBuildHasher>;
