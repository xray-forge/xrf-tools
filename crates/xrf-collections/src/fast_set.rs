use std::collections::HashSet;

use crate::fast_build_hasher::FastBuildHasher;

/// A set in no particular order, hashed with [`FastBuildHasher`]. Build one with `FastSet::default()`.
pub type FastSet<T> = HashSet<T, FastBuildHasher>;
