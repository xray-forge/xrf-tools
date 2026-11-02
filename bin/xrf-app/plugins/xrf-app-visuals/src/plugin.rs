use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime};
use xrf_app_core::session::SessionScopes;

use crate::{VisualBrowses, VisualModels, VisualPoses};

/// The `visuals` plugin: models opened with their motions, the roots browsed for them, and a model shown in a viewport.
pub struct VisualsPlugin;

impl VisualsPlugin {
  /// The plugin, its models, poses and browses managed as it sets up.
  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    crate::ipc::visuals::builder()
      .setup(|application, _| {
        let scopes = application.state::<SessionScopes>();

        application.manage(VisualModels::new("visual model", &scopes));
        application.manage(VisualPoses::new("visual pose", &scopes));
        application.manage(VisualBrowses::new("visual browse", &scopes));

        Ok(())
      })
      .build()
  }
}
