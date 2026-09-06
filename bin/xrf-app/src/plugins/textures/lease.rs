//! What a textures job registers itself as, what it holds exclusively, and why.

use std::path::Path;

use crate::core::jobs::to_comparable_path;

/// Writing a node's descriptor, its base texture, or both.
///
/// The frontend spells each of these in `EJobKind`, which is the wire contract this side owns.
pub const SAVE_JOB_KIND: &str = "textures.save";

/// Generating the `_bump` and `_bump#` pair a bumped surface binds.
pub const MAKE_BUMP_JOB_KIND: &str = "textures.make-bump";

/// Rebuilding a texture from a source image, the way its descriptor says to.
pub const BUILD_JOB_KIND: &str = "textures.build";

/// Weighing every candidate format against one texture.
pub const COMPARE_ENCODINGS_JOB_KIND: &str = "textures.compare-encodings";

/// The group every run that spends the pool on an encode holds.
///
/// One group across three kinds rather than a group each, because they compete for the same thing: each saturates the
/// execution pool for its whole duration - measured on a 2048 square texture, a bump pair is about 1.9 seconds and a
/// five-candidate comparison about 1.7, of which BC7 alone is 1.6. Two at once do not finish in the time one takes;
/// they finish in the time both take, having also made the machine unresponsive in between. Serializing them is what
/// the pool would do anyway, and this way a surface can say so instead of appearing to hang.
pub const TEXTURE_ENCODE_GROUP: &str = "textures.encode";

/// Phase a comparison reports while it weighs one candidate after another.
///
/// Here rather than in `xrf-texture`'s vocabulary because the comparison is the app's own loop: the crate encodes one
/// candidate and knows nothing about the five.
pub const TEXTURE_PHASE_WEIGH: &str = "weigh";

/// One file a run would write, as a lease key.
///
/// Keyed by the path alone under one prefix shared by every kind that writes, rather than by `<kind>:<path>` as a tool
/// with only one writer can afford. Three of these kinds write textures - a save replacing a base, a build rebuilding
/// one, a generation writing a pair - and any two of them aimed at one file are the collision worth refusing. A key
/// carrying the kind would only ever have made a run collide with another run of its own kind, which is the case that
/// matters least.
pub fn to_texture_lease_key(path: &Path) -> String {
  format!("textures:{}", to_comparable_path(path))
}
