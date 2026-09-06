use std::sync::{Arc, Mutex};

use xrf_vfs::XrayRoots;

use crate::plugins::textures::encoding::TextureEncodingSession;

/// What the textures tool is holding between calls.
pub struct TextureState {
  /// The roots the explorer is browsing, or `None` when nothing is open.
  pub opened: Mutex<Option<XrayRoots>>,
  /// The encodes the last format comparison produced, or `None` when none has run since the tool opened.
  ///
  /// One session rather than a map keyed by texture: the editor holds one node at a time, and a comparison is
  /// megabytes of block data that nobody wants accumulating behind a person browsing a tree. A new comparison replaces
  /// it whole, which is what makes an encode belonging to a texture nobody is looking at any more unreachable from a
  /// later save.
  pub encodings: Arc<Mutex<Option<TextureEncodingSession>>>,
}

impl TextureState {
  pub fn new() -> Self {
    Self {
      opened: Mutex::new(None),
      encodings: Arc::new(Mutex::new(None)),
    }
  }
}
