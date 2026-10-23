//! The composition root: what the application is assembled from, and what is in place before its first command runs.

use std::error::Error;
use std::process::ExitCode;
use std::sync::Arc;

use tauri::{App, AppHandle, Builder, Manager, RunEvent, Runtime, Window, WindowEvent, Wry};
use xrf_job::ExecutionRequest;

use crate::core::app_directories::AppDirectories;
use crate::core::assets::AssetMountState;
use crate::core::directories::get_local_data_directory;
use crate::core::execution::ExecutionState;
use crate::core::jobs::JobRegistry;
use crate::core::logging::setup_logger;
use crate::core::process::process_uptime;
use crate::core::session::SessionScopes;
use crate::core::transport::{TransportEndpoint, TransportOrigins, TransportServer, TransportToken};
use crate::core::window::{WindowCloseApprovals, WindowHandles, build_main_window};
use crate::ipc::registry::transport_routes;
use crate::plugins::registry::domain_plugins;
use crate::plugins::system::diagnostics::log_startup;

/// Assemble the application from its plugins and hand control to Tauri, logging from the moment its configuration
/// names where; a failure before any window is said aloud and ends the process with failure.
pub fn run() -> ExitCode {
  let context = tauri::generate_context!();

  setup_logger(&context.config().identifier);
  log::info!("Starting application");
  log_startup();

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

  let builder: Builder<Wry> = assemble(builder, execution, directories).setup(|application| {
    manage_shared_state(application)?;
    start_transport(application)?;
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

/// The domain plugins over the state every one's setup builds on, managed before any of them: the pool, the asset state
/// every setup hands the sessions a commit moves along, the scopes every session's handles are released by, and where
/// the application writes. A window that goes releases every scope its pages held.
fn assemble<R: Runtime>(builder: Builder<R>, execution: ExecutionState, directories: AppDirectories) -> Builder<R> {
  let builder: Builder<R> = builder
    .manage(execution)
    .manage(AssetMountState::new())
    .manage(SessionScopes::default())
    .manage(directories)
    .on_window_event(|window: &Window<R>, event: &WindowEvent| {
      if matches!(event, WindowEvent::Destroyed) {
        let released: usize = window.state::<SessionScopes>().release_window(window.label());

        log::info!(
          "Window '{}' went, releasing the {released} opening(s) its pages held",
          window.label()
        );
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

/// Starts the loopback transport the bulk routes are served from, before the window can ask where it listens.
fn start_transport(application: &mut App) -> Result<(), Box<dyn Error>> {
  let token: TransportToken = TransportToken::generate()?;
  let server: TransportServer<AppHandle> = TransportServer::new(
    transport_routes(),
    TransportOrigins::of_config(application.config()),
    token.clone(),
    application.handle().clone(),
  );

  let (address, serving) = server.bind()?;

  tauri::async_runtime::spawn(serving);

  log::info!("Transport listening on {address}");

  application.manage(TransportEndpoint {
    origin: format!("http://{address}"),
    token: token.get_value().to_string(),
  });

  Ok(())
}

#[cfg(test)]
mod tests {
  use std::num::NonZeroUsize;

  use tauri::test::{MockRuntime, mock_builder, mock_context, noop_assets};
  use tauri::{App, Manager};
  use xrf_job::ExecutionRequest;

  use super::assemble;
  use crate::core::app_directories::AppDirectories;
  use crate::core::execution::ExecutionState;
  use crate::plugins::levels::shown_levels::ShownLevels;
  use crate::plugins::levels::state::LevelState;
  use crate::plugins::render::render_viewports::RenderViewports;
  use crate::plugins::render::renderer::Renderer;
  use crate::plugins::render::texture_reloads::TextureReloads;
  use crate::plugins::render::viewport_shows::ViewportShows;
  use crate::plugins::render::world_state::WorldState;

  /// A plugin whose setup misses what it builds on fails the build, so this is where an order or a unit gone wrong
  /// shows, before any command asks.
  #[test]
  fn every_plugin_sets_up_over_the_state_managed_before_it() {
    let execution: ExecutionState =
      ExecutionState::new(ExecutionRequest::Workers(NonZeroUsize::new(2).expect("two workers"))).expect("pool starts");
    let application: App<MockRuntime> = assemble(mock_builder(), execution, AppDirectories::new(None))
      .build(mock_context(noop_assets()))
      .expect("every plugin to set up");

    assert!(application.try_state::<AppDirectories>().is_some());
    assert!(application.try_state::<LevelState>().is_some());
    assert!(application.try_state::<Renderer>().is_some());
    assert!(application.try_state::<WorldState>().is_some());
    assert!(application.try_state::<ViewportShows>().is_some());
    assert!(application.try_state::<TextureReloads>().is_some());
    assert!(application.try_state::<ShownLevels>().is_some());

    application.state::<RenderViewports>().shut_down();
  }
}
