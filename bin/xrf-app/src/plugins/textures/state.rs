use std::sync::Mutex;

use xrf_vfs::XrayRoots;

/// The roots the textures explorer is browsing, or `None` when nothing is open.
pub struct TextureState {
  pub opened: Mutex<Option<XrayRoots>>,
}

impl TextureState {
  pub fn new() -> Self {
    Self {
      opened: Mutex::new(None),
    }
  }
}
