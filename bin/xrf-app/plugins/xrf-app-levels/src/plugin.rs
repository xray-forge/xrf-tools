use tauri::plugin::TauriPlugin;
use tauri::webview::{PageLoadEvent, PageLoadPayload};
use tauri::{AppHandle, Manager, RunEvent, Runtime, State, Webview, WindowEvent};
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::SessionScopes;
use xrf_app_render::WorldState;
use xrf_audio::AudioServer;
use xrf_level_source::{LevelRenderParts, OpenedLevel};

use crate::level::SelectedLevels;
use crate::rawdata::level_compiler_libraries::LevelCompilerLibraries;
use crate::render::{LevelSceneFollower, ShownLevels};
use crate::scene::{OpenedScene, SelectedScenes, ShownScenes};
use crate::viewer::recording::LevelRecordings;

/// The `levels` plugin: compiled levels and Level Editor scenes opened as documents and shown in viewports, with the
/// viewer's capture and export. A window's recording is released when its page reloads or the window closes.
pub struct LevelsPlugin;

impl LevelsPlugin {
  /// Build the plugin; its setup manages the level and scene documents and their shown sources.
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
            |current, assets| OpenedLevel::derive(current, assets.get_host()).map_err(|error| error.to_string()),
          )
          .with_view_parts(LevelRenderParts::VIEW_PARTS);
        // A scene records its inputs and is derived again on the pool as a level is.
        let scenes: SelectedScenes = SelectedScenes::new("scene", &application.state::<SessionScopes>())
          .with_deferred_derivation(
            &assets,
            application.state::<ExecutionState>().get_pool(),
            OpenedScene::derive,
          );
        // A shown level is heard through the audio plugin's server, set up before the domains.
        let shown: ShownLevels = ShownLevels::new(application.state::<AudioServer>().inner().clone());

        // A shown level's scene follows its session.
        assets.add_subscriber(LevelSceneFollower {
          levels: levels.clone(),
          shown: shown.clone(),
          world,
        });
        // todo: follow a shown scene's session through commits as `LevelSceneFollower` follows a level's, reloading the
        // models whose library objects a commit changed.

        application.manage(levels);
        application.manage(scenes);
        application.manage(shown);
        application.manage(ShownScenes::default());
        application.manage(LevelRecordings::default());
        application.manage(LevelCompilerLibraries::locate(
          application.path().resource_dir().ok().as_deref(),
        ));

        Ok(())
      })
      .on_page_load(Self::on_page_load)
      .on_event(Self::on_event)
      .build()
  }

  /// Release a window's recording as a page starts loading, since the page that would stop it is gone.
  fn on_page_load<R: Runtime>(webview: &Webview<R>, payload: &PageLoadPayload<'_>) {
    if payload.event() == PageLoadEvent::Started {
      Self::release_window(webview.app_handle(), webview.window().label());
    }
  }

  /// Release a destroyed window's recording.
  fn on_event<R: Runtime>(application: &AppHandle<R>, event: &RunEvent) {
    if let RunEvent::WindowEvent {
      label,
      event: WindowEvent::Destroyed,
      ..
    } = event
    {
      Self::release_window(application, label);
    }
  }

  fn release_window<R: Runtime>(application: &AppHandle<R>, window: &str) {
    if let (Some(recordings), Some(world)) = (
      application.try_state::<LevelRecordings>(),
      application.try_state::<WorldState>(),
    ) {
      recordings.release_window(window, &world);
    }
  }
}
