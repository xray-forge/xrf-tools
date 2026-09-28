//! The composition root: what the application is assembled from, and what is in place before its first command runs.

use std::error::Error;
use std::sync::Arc;

use tauri::{App, AppHandle, Builder, Manager, Wry};
use xrf_job::ExecutionRequest;

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::core::jobs::JobRegistry;
use crate::core::transport::{TransportEndpoint, TransportOrigins, TransportServer, TransportToken};
use crate::core::window::build_main_window;
use crate::ipc::registry::transport_routes;
use crate::plugins::registry::domain_plugins;

/// Assemble the application from its plugins and hand control to Tauri.
pub fn run() {
  log::info!("Starting application");

  let builder: Builder<Wry> = Builder::default()
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
    .run(tauri::generate_context!())
    .expect("Error while running tauri application")
}

/// Manage the state that outlives every command and belongs to no single domain.
fn manage_shared_state(application: &mut App) -> Result<(), Box<dyn Error>> {
  application.manage(AssetMountState::new());

  application.manage(Arc::new(JobRegistry::new()).start_reporting());

  // One pool for the process.
  let execution: ExecutionState = ExecutionState::new(ExecutionRequest::Auto)?;

  log::info!(
    "Execution: {} worker(s) ({})",
    execution.get_plan().get_workers(),
    execution.get_plan().get_origin().as_str()
  );

  application.manage(execution);

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
