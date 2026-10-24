use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime};
use xrf_app_core::session::SessionScopes;

use crate::EquipmentSprites;

/// The `sprite-equipment` plugin: equipment sheets opened, previewed, reloaded and packed.
pub struct SpriteEquipmentPlugin;

impl SpriteEquipmentPlugin {
  /// The plugin, its sprites managed as it sets up.
  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    crate::ipc::sprite_equipment::builder()
      .setup(|application, _| {
        application.manage(EquipmentSprites::new(
          "equipment sprite",
          &application.state::<SessionScopes>(),
        ));

        Ok(())
      })
      .build()
  }
}
