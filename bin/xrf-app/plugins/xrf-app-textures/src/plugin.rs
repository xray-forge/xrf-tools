use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime};
use xrf_app_core::session::SessionScopes;

use crate::browse::TextureBrowses;
use crate::encoding::TextureComparisons;

/// The `textures` plugin: root sets browsed for their textures, one texture described and edited, its formats weighed,
/// and a texture laid on a body in a viewport.
pub struct TexturesPlugin;

impl TexturesPlugin {
  /// The plugin, its browses and comparisons managed as it sets up, each closed with the scope that holds it.
  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    crate::ipc::textures::builder()
      .setup(|application, _| {
        let scopes = application.state::<SessionScopes>();

        application.manage(TextureBrowses::new("texture browse", &scopes));
        application.manage(TextureComparisons::new("texture comparison", &scopes));

        Ok(())
      })
      .build()
  }
}
