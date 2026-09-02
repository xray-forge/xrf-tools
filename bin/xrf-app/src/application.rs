use std::sync::Arc;

use tauri::Manager;
use tauri::utils::config::WindowConfig;
use tauri::webview::WebviewWindowBuilder;
use xrf_job::ExecutionRequest;

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::core::jobs::JobRegistry;
use crate::core::webview_extensions::DevExtensions;
use crate::plugins::archives::plugin::ArchivesPlugin;
use crate::plugins::assets::plugin::AssetsPlugin;
use crate::plugins::configs::plugin::ConfigsPlugin;
use crate::plugins::dialogs::plugin::DialogsPlugin;
use crate::plugins::exports::plugin::ExportsPlugin;
use crate::plugins::gamedata::plugin::GamedataPlugin;
use crate::plugins::jobs::plugin::JobsPlugin;
use crate::plugins::spawn::plugin::SpawnPlugin;
use crate::plugins::sprite_equipment::plugin::SpriteEquipmentPlugin;
use crate::plugins::system::plugin::SystemPlugin;
use crate::plugins::translations::plugin::TranslationsPlugin;
use crate::plugins::visuals::plugin::VisualsPlugin;

/// Assemble the application from its plugins and hand control to Tauri.
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_shell::init())
    // Custom plugins.
    .plugin(AssetsPlugin::init())
    .plugin(ArchivesPlugin::init())
    .plugin(DialogsPlugin::init())
    .plugin(ExportsPlugin::init())
    .plugin(GamedataPlugin::init())
    .plugin(JobsPlugin::init())
    .plugin(SpawnPlugin::init())
    .plugin(ConfigsPlugin::init())
    .plugin(SpriteEquipmentPlugin::init())
    .plugin(SystemPlugin::init())
    .plugin(TranslationsPlugin::init())
    .plugin(VisualsPlugin::init())
    .setup(|app| {
      // Core state before any command can run, so every domain sharing the asset roots finds it managed.
      app.manage(AssetMountState::new());

      // Behind an `Arc` because a registration outlives the command frame that took it: the guard travels onto a
      // blocking thread and releases its leases there, which a `State` borrow cannot do.
      app.manage(Arc::new(JobRegistry::new()));

      // One pool for the process, started before any command can reach for it. `Auto` because nothing here asks the
      // user how much of their machine an application may use; the width is stated rather than chosen, and the plan is
      // what every bounded job shares rather than each taking its own.
      let execution: ExecutionState = ExecutionState::new(ExecutionRequest::Auto)?;

      // Said once at startup for the reason the CLI says it per run: the width is the first thing a report of "it was
      // slow" or "it used the whole machine" needs, and the last thing anyone thinks to record.
      log::info!(
        "Execution: {} worker(s) ({})",
        execution.get_plan().get_workers(),
        execution.get_plan().get_origin().as_str()
      );

      app.manage(execution);

      // The window stays described by tauri.conf.json with `create: false` and is built here so a
      // debug build can extend it with locally supplied extensions: their path reaches the webview
      // through the builder only, the configuration has no field carrying it.
      let window: &WindowConfig = app
        .config()
        .app
        .windows
        .first()
        .expect("Main window has to be declared in tauri.conf.json");

      WebviewWindowBuilder::from_config(app.handle(), window)?
        .with_dev_extensions()
        .build()?;

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("Error while running tauri application")
}
