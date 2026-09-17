//! The composition root: what the application is assembled from, and what is in place before its first command runs.

use std::error::Error;
use std::sync::Arc;

use tauri::utils::config::WindowConfig;
use tauri::webview::{WebviewWindow, WebviewWindowBuilder};
use tauri::{App, Builder, Manager, Wry};
use xrf_job::ExecutionRequest;

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::core::jobs::JobRegistry;
use crate::core::webview_extensions::DevExtensions;
use crate::core::window_reveal::reveal_window_on_timeout;
use crate::plugins::registry::domain_plugins;

/// Assemble the application from its plugins and hand control to Tauri.
pub fn run() {
  log::info!("Starting application");

  let builder: Builder<Wry> = Builder::default()
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_shell::init());

  log::info!("Registering domain plugins");

  let builder: Builder<Wry> = domain_plugins()
    .into_iter()
    .fold(builder, Builder::plugin)
    .setup(|application| {
      manage_shared_state(application)?;
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

/// Build the window `tauri.conf.json` describes.
fn build_main_window(application: &mut App) -> Result<(), Box<dyn Error>> {
  let config: &WindowConfig = application
    .config()
    .app
    .windows
    .first()
    .expect("Main window has to be declared in tauri.conf.json");

  let window: WebviewWindow = WebviewWindowBuilder::from_config(application.handle(), config)?
    .with_dev_extensions()
    .build()?;

  // The configuration declares it hidden: the document reveals itself once it has resolved its colour scheme.
  reveal_window_on_timeout(window);

  log::info!("Built main window");

  Ok(())
}
