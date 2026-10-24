use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime, State};
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::SessionScopes;
use xrf_app_render::WorldState;

use crate::level::{OpenedLevel, SelectedLevels};
use crate::render::{LevelRenderSource, LevelSceneFollower, ShownLevels};

/// The `levels` plugin: compiled levels opened as documents whose parts follow commits, shown in a viewport, and
/// captured or exported through the viewer.
pub struct LevelsPlugin;

impl LevelsPlugin {
  /// The plugin, its levels and shown sources managed as it sets up.
  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    crate::ipc::levels::builder()
      .setup(|application, _| {
        let assets: State<'_, AssetMountState> = application.state::<AssetMountState>();
        let world: WorldState = application
          .try_state::<WorldState>()
          .ok_or("The render plugin is set up before the levels plugin, whose shown levels its world draws")?
          .inner()
          .clone();
        // An open level records what it and its parts read, so a commit carries the parts it did not touch and reads
        // the level again on the pool where it moved one of its own files, which takes seconds no save waits for.
        let levels: SelectedLevels = SelectedLevels::new("level", &application.state::<SessionScopes>())
          .with_deferred_derivation(
            &assets,
            application.state::<ExecutionState>().get_pool(),
            OpenedLevel::derive,
          )
          .with_view_parts(LevelRenderSource::VIEW_PARTS);
        let shown: ShownLevels = ShownLevels::default();

        // A shown level's scene follows its session.
        assets.add_subscriber(LevelSceneFollower {
          levels: levels.clone(),
          shown: shown.clone(),
          world,
        });

        application.manage(levels);
        application.manage(shown);

        Ok(())
      })
      .build()
  }
}
