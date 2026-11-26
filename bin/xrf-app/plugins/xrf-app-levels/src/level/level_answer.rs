use std::convert::Infallible;
use std::fmt::Display;

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
  /// What `read` answers of the level `session_id` holds, its failure as the page shows it.
  ///
  /// # Errors
  ///
  /// Returns an error when no level is held under `session_id`, its roots cannot be searched, or `read` fails.
  pub(crate) async fn read<U: Send + Sync + 'static, E: Display>(
    levels: &SelectedLevels,
    assets: &AssetMountState,
    execution: &ExecutionState,
    session_id: SessionId,
    what: &str,
    read: impl FnOnce(&SessionSnapshot<SelectedLevel>, &XraySearch) -> Result<U, E> + Send + 'static,
  ) -> TauriResult<SessionSnapshot<U>> {
    let assets: AssetMountState = AssetMountState::clone(assets);

    levels
      .read_on_pool(execution, session_id, what, move |current| {
        read(current, &assets.search(&current.roots)?).map_err(|error| error.to_string())
      })
      .await
  }

  /// What `get` answers of the level `session_id` holds, a read that cannot fail once the roots are searched.
  ///
  /// # Errors
  ///
  /// Returns an error when no level is held under `session_id` or its roots cannot be searched.
  pub(crate) async fn get<U: Send + Sync + 'static>(
    levels: &SelectedLevels,
    assets: &AssetMountState,
    execution: &ExecutionState,
    session_id: SessionId,
    what: &str,
    get: impl FnOnce(&SessionSnapshot<SelectedLevel>, &XraySearch) -> U + Send + 'static,
  ) -> TauriResult<SessionSnapshot<U>> {
    Self::read(levels, assets, execution, session_id, what, move |current, search| {
      Ok::<U, Infallible>(get(current, search))
    })
    .await
  }
}
