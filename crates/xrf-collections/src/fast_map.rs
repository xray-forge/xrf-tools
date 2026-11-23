use std::collections::HashMap;

use crate::fast_build_hasher::FastBuildHasher;

/// A map in no particular order, hashed with [`FastBuildHasher`]. Build one with `FastMap::default()`; `new` belongs
/// to the standard library's own hasher.
pub type FastMap<K, V> = HashMap<K, V, FastBuildHasher>;
