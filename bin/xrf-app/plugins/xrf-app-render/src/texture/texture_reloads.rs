use std::path::Path;

use xrf_vfs::XrayRoots;

use crate::Renderer;
use crate::texture::texture_watcher::TextureWatcher;

/// Keeps every drawn texture current with its file: the roots shown sources read are watched for texture files written
/// there by anything, and a file this application wrote elsewhere is reloaded as its writer says.
pub struct TextureReloads {
  watcher: TextureWatcher,
  renderer: Renderer,
}

impl TextureReloads {
  /// Reloads into what `renderer` draws, watching no root yet.
  pub fn new(renderer: Renderer) -> Self {
    Self {
      watcher: TextureWatcher::new(renderer.get_queue()),
      renderer,
    }
  }

  /// Watches the roots a shown source reads for texture files written there.
  pub fn watch(&self, roots: &XrayRoots) {
    self.watcher.watch(roots);
  }

  /// Loads again wherever they are drawn the textures among files this application wrote, by their paths: those
  /// outside the roots watched, as the watcher reloads the rest itself.
  pub fn reload_written<T: AsRef<Path>>(&self, paths: &[T]) {
    let names: Vec<String> = paths
      .iter()
      .map(AsRef::as_ref)
      .filter(|path| !self.watcher.is_watched(path))
      .filter_map(TextureWatcher::to_texture_name)
      .collect();

    self.renderer.reload_textures(names);
  }
}
