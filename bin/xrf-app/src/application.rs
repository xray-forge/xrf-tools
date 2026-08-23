use tauri::Manager;
use tauri::utils::config::WindowConfig;
use tauri::webview::WebviewWindowBuilder;

use crate::core::assets::AssetWorldState;
use crate::core::webview_extensions::DevExtensions;
use crate::plugins::archives::plugin::ArchivesPlugin;
use crate::plugins::assets::plugin::AssetsPlugin;
use crate::plugins::configs::plugin::ConfigsPlugin;
use crate::plugins::dialogs::plugin::DialogsPlugin;
use crate::plugins::equipment_icons::plugin::EquipmentIconsPlugin;
use crate::plugins::exports::plugin::ExportsPlugin;
use crate::plugins::spawn::plugin::SpawnPlugin;
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
    .plugin(SpawnPlugin::init())
    .plugin(ConfigsPlugin::init())
    .plugin(EquipmentIconsPlugin::init())
    .plugin(SystemPlugin::init())
    .plugin(TranslationsPlugin::init())
    .plugin(VisualsPlugin::init())
    .setup(|app| {
      // Core state before any command can run, so every domain sharing the asset world finds it managed.
      app.manage(AssetWorldState::new());

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
