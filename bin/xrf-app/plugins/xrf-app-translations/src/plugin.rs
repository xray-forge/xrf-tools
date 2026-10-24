use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime};
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::SessionScopes;

use crate::{TranslationProjectRead, TranslationProjects};

/// The `translations` plugin: translations projects edited and saved, followed through commits, and verified,
/// formatted, built and imported as jobs.
pub struct TranslationsPlugin;

impl TranslationsPlugin {
  /// The plugin, its projects managed as it sets up.
  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    crate::ipc::translations::builder()
      .setup(|application, _| {
        // An open project records every file it read and reads them again when a commit moved one: a save among them.
        application.manage(
          TranslationProjects::new("translations project", &application.state::<SessionScopes>())
            .with_derivation(&application.state::<AssetMountState>(), TranslationProjectRead::derive),
        );

        Ok(())
      })
      .build()
  }
}
