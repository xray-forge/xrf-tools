//! The composition root: what the application is assembled from, and what is in place before its first command runs.

use std::error::Error;
use std::process::ExitCode;
use std::sync::Arc;

use tauri::{App, AppHandle, Builder, Manager, RunEvent, Wry};
use xrf_job::ExecutionRequest;

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::core::jobs::JobRegistry;
use crate::core::logging::setup_logger;
use crate::core::process::process_uptime;
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

  let builder: Builder<Wry> = Builder::default()
    .manage(execution)
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_store::Builder::default().build());

  log::info!("Registering domain plugins");

  let builder: Builder<Wry> = domain_plugins()
    .into_iter()
    .fold(builder, Builder::plugin)
    .setup(|application| {
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
  application.manage(AssetMountState::new());
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
