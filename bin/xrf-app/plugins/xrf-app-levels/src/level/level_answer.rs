use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_level_source::SelectedLevel;
use xrf_vfs::XraySearch;

use crate::level::SelectedLevels;

/// How every query of an open level is answered: on the application's pool, through a search of the roots the level
/// was opened in, at its snapshot's revision.
pub(crate) struct LevelAnswer;

impl LevelAnswer {
  /// What `read` answers of the level `session_id` holds.
  ///
  /// # Errors
  ///
  /// Returns an error when no level is held under `session_id`, its roots cannot be searched, or `read` fails.
  pub(crate) async fn read<U: Send + Sync + 'static>(
    levels: &SelectedLevels,
    assets: &AssetMountState,
    execution: &ExecutionState,
    session_id: SessionId,
    what: &str,
    read: impl FnOnce(&SessionSnapshot<SelectedLevel>, &XraySearch) -> TauriResult<U> + Send + 'static,
  ) -> TauriResult<SessionSnapshot<U>> {
    let assets: AssetMountState = AssetMountState::clone(assets);

    levels
      .read_on_pool(execution, session_id, what, move |current| {
        read(current, &assets.search(&current.roots)?)
      })
      .await
  }
}
