use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use specta::Types;

use crate::ipc::bindings::command_module::{export_raw_commands, finalize_command_module};
use crate::ipc::bindings::constants::{COMMANDS_DIRECTORY, TYPES_DIRECTORY};
use crate::ipc::bindings::exporter::command_exporter;
use crate::ipc::bindings::output::reset_directory;
use crate::ipc::bindings::ownership::TypeOwnership;
use crate::ipc::bindings::types_module::export_type_modules;
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

/// Every Tauri plugin whose typed commands are mirrored, as `(plugin name, Specta builder)`.
///
/// A domain with no typed command at all would be absent: there would be no typed module to write.
fn command_modules<R: tauri::Runtime>() -> Vec<(&'static str, tauri_specta::Builder<R>)> {
  vec![
    // The raw `assets` read is absent from this builder by construction, and is generated beside it.
    (AssetsPlugin::NAME, AssetsPlugin::specta_builder::<R>()),
    (ArchivesPlugin::NAME, ArchivesPlugin::specta_builder::<R>()),
    (ConfigsPlugin::NAME, ConfigsPlugin::specta_builder::<R>()),
    (DialogsPlugin::NAME, DialogsPlugin::specta_builder::<R>()),
    (EquipmentIconsPlugin::NAME, EquipmentIconsPlugin::specta_builder::<R>()),
    (ExportsPlugin::NAME, ExportsPlugin::specta_builder::<R>()),
    (SpawnPlugin::NAME, SpawnPlugin::specta_builder::<R>()),
    (SystemPlugin::NAME, SystemPlugin::specta_builder::<R>()),
    (TranslationsPlugin::NAME, TranslationsPlugin::specta_builder::<R>()),
    // The raw `visuals` commands are absent from this builder by construction, and are generated beside it.
    (VisualsPlugin::NAME, VisualsPlugin::specta_builder::<R>()),
  ]
}

#[test]
fn export_typescript_bindings() {
  let output: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../xrf-ui/src/core/bindings");
  let types_output: PathBuf = output.join(TYPES_DIRECTORY);
  let commands_output: PathBuf = output.join(COMMANDS_DIRECTORY);

  reset_directory(&types_output);
  reset_directory(&commands_output);

  let collected: Arc<Mutex<Types>> = Arc::new(Mutex::new(Types::default()));
  let plugins: Vec<(&str, tauri_specta::Builder<tauri::Wry>)> = command_modules();

  for (plugin, builder) in &plugins {
    builder
      .export(
        command_exporter(Arc::clone(&collected)),
        commands_output.join(format!("{plugin}.ts")),
      )
      .unwrap_or_else(|error| panic!("Failed to export {plugin} commands: {error}"));
  }

  let collected: Types = Arc::try_unwrap(collected)
    .unwrap_or_else(|_| panic!("Collected types are still borrowed"))
    .into_inner()
    .expect("Collected types lock is poisoned");
  let ownership: TypeOwnership = export_type_modules(&types_output, &collected);

  for (plugin, _) in &plugins {
    finalize_command_module(&commands_output.join(format!("{plugin}.ts")), plugin, &ownership);
  }

  export_raw_commands(
    &commands_output.join(format!("{}-raw.ts", crate::ipc::registry::assets::NAME)),
    crate::ipc::registry::assets::NAME,
    crate::ipc::registry::assets::RAW_COMMANDS,
    &ownership,
  );

  export_raw_commands(
    &commands_output.join(format!("{}-raw.ts", crate::ipc::registry::visuals::NAME)),
    crate::ipc::registry::visuals::NAME,
    crate::ipc::registry::visuals::RAW_COMMANDS,
    &ownership,
  );
}
