use tauri::plugin::{Builder, TauriPlugin};
use tauri::{Manager, Runtime, State};
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::SessionScopes;
use xrf_app_render::world_state::WorldState;

use crate::level_scene_follower::LevelSceneFollower;
use crate::shown_levels::ShownLevels;
use crate::state::LevelState;

pub struct LevelsPlugin {}

impl LevelsPlugin {
  pub const NAME: &'static str = crate::registry::levels::NAME;

  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    log::info!("Initialize plugin {}", Self::NAME);

    Builder::new(Self::NAME)
      .setup(|application, _| {
        let state: LevelState = LevelState::new(
          application.state::<ExecutionState>().get_pool(),
          &application.state::<SessionScopes>(),
        );
        let shown: ShownLevels = ShownLevels::default();
        let world: WorldState = application
          .try_state::<WorldState>()
          .ok_or("The render plugin is set up before the levels plugin, whose shown levels its world draws")?
          .inner()
          .clone();
        let assets: State<'_, AssetMountState> = application.state::<AssetMountState>();

        // An open level records what it and its parts read, so a commit carries the parts it did not touch and reads
        // the level again where it moved one of its own files; a shown one's scene follows it.
        assets.add_follower(state.levels.clone());
        assets.add_subscriber(LevelSceneFollower {
          levels: state.levels.clone(),
          shown: shown.clone(),
          world,
        });

        application.manage(state);
        application.manage(shown);

        Ok(())
      })
      .invoke_handler(xrf_app_core::logging::warn_on_unhandled_command(
        Self::NAME,
        crate::registry::levels::handler(),
      ))
      .build()
  }

  #[cfg(feature = "typescript-bindings")]
  pub fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    crate::registry::levels::specta_builder()
  }
}
