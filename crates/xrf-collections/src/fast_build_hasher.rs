/// The hasher every map and set in the workspace builds with: `rustc-hash`'s Fx, several times faster than the
/// standard library's SipHash on the short names game files are keyed by.
///
/// Fx gives up SipHash's resistance to keys chosen to collide. Every key here comes from a file the user opened or a
/// name the tools made, never from a network peer, so nothing chooses them against the hasher. A re-export rather than
/// an alias, so the unit value builds a map (`with_capacity_and_hasher(n, FastBuildHasher)`) and hashes one key
/// (`FastBuildHasher.hash_one(key)`).
pub use rustc_hash::FxBuildHasher as FastBuildHasher;
