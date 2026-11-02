use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime};
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::SessionScopes;

use crate::browse::{ArchiveSubject, ArchiveSubjects};

/// The `archives` plugin: volume sets and game folders browsed, read, described and extracted, and trees packed,
/// patched and unpacked as jobs.
pub struct ArchivesPlugin;

impl ArchivesPlugin {
  /// The plugin, its subjects managed as it sets up: a world listed again on the pool once a commit moved what it read.
  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    crate::ipc::archives::builder()
      .setup(|application, _| {
        application.manage(
          ArchiveSubjects::new("archive subject", &application.state::<SessionScopes>()).with_deferred_derivation(
            &application.state::<AssetMountState>(),
            application.state::<ExecutionState>().get_pool(),
            ArchiveSubject::derive,
          ),
        );

        Ok(())
      })
      .build()
  }
}
