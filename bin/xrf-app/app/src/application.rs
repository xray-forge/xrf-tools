//! The composition root: what the application is assembled from, and what is in place before its first command runs.

use std::error::Error;
use std::process::ExitCode;
use std::sync::Arc;

use tauri::{App, Builder, Manager, RunEvent, Runtime, Window, WindowEvent, Wry};
use xrf_app_core::app_directories::AppDirectories;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::directories::get_local_data_directory;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::JobRegistry;
use xrf_app_core::logging::setup_logger;
use xrf_app_core::process::process_uptime;
use xrf_app_core::session::SessionScopes;
use xrf_app_core::transport::TransportServer;
use xrf_app_core::window::{WindowCloseApprovals, WindowHandles};
use xrf_app_system::diagnostics::log_startup;
use xrf_build_info::{BuildInfo, build_info};
use xrf_job::ExecutionRequest;

use crate::ipc::registry::transport_routes;
use crate::plugins::domain_plugins;
use crate::window::build_main_window;

/// Assemble the application from its plugins and hand control to Tauri, logging from the moment its configuration
/// names where; a failure before any window is said aloud and ends the process with failure.
pub fn run() -> ExitCode {
  let context = tauri::generate_context!();

  // Recorded by this binary's build script, so it describes the binary rather than whichever crate reads it.
  let build: BuildInfo = build_info!();

  setup_logger(&context.config().identifier, build.version);
  log::info!("Starting application");
  log_startup(&build);

  // One pool for the process, managed before any plugin's setup, since the renderer's loaders run on it too.
  let execution: ExecutionState = match ExecutionState::new(ExecutionRequest::Auto) {
    Ok(execution) => execution,
    Err(error) => {
      let message: String = format!("Cannot start the application's worker pool: {error}");

      log::error!("{message}");
      show_start_failure(&message);

      return ExitCode::FAILURE;
    }
  };

  log::info!(
    "Execution: {} worker(s) ({})",
    execution.get_plan().get_workers(),
    execution.get_plan().get_origin().as_str()
  );

  let directories: AppDirectories = AppDirectories::new(get_local_data_directory(&context.config().identifier));
  let builder: Builder<Wry> = Builder::default()
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_store::Builder::default().build());

  log::info!("Registering domain plugins");

  let builder: Builder<Wry> = assemble(builder, build, execution, directories).setup(|application| {
    manage_shared_state(application)?;
    // Before the window, which asks where the transport listens as its page loads.
    TransportServer::start(application, transport_routes())?;
    build_main_window(application)?;

    Ok(())
  });

  builder
    .build(context)
    .expect("Error while building tauri application")
    .run(|_, event| match event {
      RunEvent::ExitRequested { code, .. } => log::info!("Application exit requested, code {code:?}"),
      // The last line a log gets, so one that ends without it ended without the application exiting.
      RunEvent::Exit => log::info!("Exiting application after {:.1?}", process_uptime()),
      _ => {}
    });

  ExitCode::SUCCESS
}

/// The domain plugins over the state every one's setup builds on, managed before any of them: which build is running,
/// the pool, the asset state every setup hands the sessions a commit moves along, the scopes every session's handles
/// are released by, and where the application writes. A window that goes releases every scope its pages held.
fn assemble<R: Runtime>(
  builder: Builder<R>,
  build: BuildInfo,
  execution: ExecutionState,
  directories: AppDirectories,
) -> Builder<R> {
  // Closed documents are dropped on the pool, so neither a command nor the event loop waits for a level to go.
  let scopes: SessionScopes = SessionScopes::new(execution.get_pool());
  let builder: Builder<R> = builder
    .manage(build)
    .manage(execution)
    .manage(AssetMountState::new())
    .manage(scopes)
    .manage(directories)
    .on_window_event(|window: &Window<R>, event: &WindowEvent| {
      if matches!(event, WindowEvent::Destroyed) {
        let label: &str = window.label();
        let released: usize = window.state::<SessionScopes>().release_window(label);

        log::info!("Window '{label}' went, releasing the {released} opening(s) its pages held");
      }
    });

  domain_plugins().into_iter().fold(builder, Builder::plugin)
}

/// Tells the user of a start that failed before any window could: a release build on Windows has no console to say it
/// in, only a log file the user does not know to open.
fn show_start_failure(message: &str) {
  #[cfg(all(windows, not(debug_assertions)))]
  {
    use windows::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW};
    use windows::core::HSTRING;

    // SAFETY: both strings outlive the call, and it names no owner window.
    unsafe {
      MessageBoxW(
        None,
        &HSTRING::from(message),
        &HSTRING::from("The application cannot start"),
        MB_OK | MB_ICONERROR,
      )
    };
  }

  #[cfg(not(all(windows, not(debug_assertions))))]
  let _ = message;
}

/// Manage the state that outlives every command and belongs to no single domain.
fn manage_shared_state(application: &mut App) -> Result<(), Box<dyn Error>> {
  application.manage(WindowHandles::default());
  application.manage(WindowCloseApprovals::default());

  application.manage(Arc::new(JobRegistry::new()).start_reporting());

  Ok(())
}

#[cfg(test)]
mod tests {
  use std::num::NonZeroUsize;

  use tauri::test::{MockRuntime, mock_builder, mock_context, noop_assets};
  use tauri::{App, Manager};
  use xrf_app_archives::browse::ArchiveBrowseState;
  use xrf_app_configs::state::ConfigsState;
  use xrf_app_core::app_directories::AppDirectories;
  use xrf_app_core::assets::AssetMountState;
  use xrf_app_core::execution::ExecutionState;
  use xrf_app_core::session::SessionScopes;
  use xrf_app_dialogs::state::DialogProjectState;
  use xrf_app_exports::state::ExportsProjectState;
  use xrf_app_levels::shown_levels::ShownLevels;
  use xrf_app_levels::state::LevelState;
  use xrf_app_render::render_viewports::RenderViewports;
  use xrf_app_render::renderer::Renderer;
  use xrf_app_render::texture_reloads::TextureReloads;
  use xrf_app_render::viewport_shows::ViewportShows;
  use xrf_app_render::world_state::WorldState;
  use xrf_app_spawn::state::SpawnFileState;
  use xrf_app_sprite_equipment::state::EquipmentSpriteState;
  use xrf_app_system::diagnostics::MachineProbeState;
  use xrf_app_textures::state::TextureState;
  use xrf_app_translations::state::TranslationProjectState;
  use xrf_app_visuals::state::VisualState;
  use xrf_build_info::{BuildInfo, build_info};
  use xrf_job::ExecutionRequest;

  use super::assemble;

  /// A plugin whose setup misses what it builds on fails the build, and a unit no setup manages fails here, so an order
  /// or a unit gone wrong shows before any command asks. What the running application manages beside them
  /// (`manage_shared_state`) is not built here.
  #[test]
  fn every_plugin_sets_up_over_the_state_managed_before_it() {
    let execution: ExecutionState =
      ExecutionState::new(ExecutionRequest::Workers(NonZeroUsize::new(2).expect("two workers"))).expect("pool starts");
    let application: App<MockRuntime> = assemble(mock_builder(), build_info!(), execution, AppDirectories::new(None))
      .build(mock_context(noop_assets()))
      .expect("every plugin to set up");

    assert!(application.try_state::<BuildInfo>().is_some());
    assert!(application.try_state::<ExecutionState>().is_some());
    assert!(application.try_state::<AssetMountState>().is_some());
    assert!(application.try_state::<SessionScopes>().is_some());
    assert!(application.try_state::<AppDirectories>().is_some());
    assert!(application.try_state::<Renderer>().is_some());
    assert!(application.try_state::<WorldState>().is_some());
    assert!(application.try_state::<ViewportShows>().is_some());
    assert!(application.try_state::<RenderViewports>().is_some());
    assert!(application.try_state::<TextureReloads>().is_some());
    assert!(application.try_state::<LevelState>().is_some());
    assert!(application.try_state::<ShownLevels>().is_some());
    assert!(application.try_state::<VisualState>().is_some());
    assert!(application.try_state::<TextureState>().is_some());
    assert!(application.try_state::<ConfigsState>().is_some());
    assert!(application.try_state::<TranslationProjectState>().is_some());
    assert!(application.try_state::<ArchiveBrowseState>().is_some());
    assert!(application.try_state::<ExportsProjectState>().is_some());
    assert!(application.try_state::<DialogProjectState>().is_some());
    assert!(application.try_state::<SpawnFileState>().is_some());
    assert!(application.try_state::<EquipmentSpriteState>().is_some());
    assert!(application.try_state::<MachineProbeState>().is_some());

    application.state::<RenderViewports>().shut_down();
  }
}
