use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use specta::Types;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::ipc::bindings::command_module::{export_raw_commands, finalize_command_module};
use crate::ipc::bindings::constants::{COMMANDS_DIRECTORY, TYPES_DIRECTORY};
use crate::ipc::bindings::exporter::command_exporter;
use crate::ipc::bindings::output::reset_directory;
use crate::ipc::bindings::ownership::TypeOwnership;
use crate::ipc::bindings::surface::{SurfaceDrift, compare_surfaces, read_surface};
use crate::ipc::bindings::types_module::export_type_modules;
use crate::plugins::archives::plugin::ArchivesPlugin;
use crate::plugins::assets::plugin::AssetsPlugin;
use crate::plugins::configs::plugin::ConfigsPlugin;
use crate::plugins::dialogs::plugin::DialogsPlugin;
use crate::plugins::equipment_icons::plugin::EquipmentIconsPlugin;
use crate::plugins::exports::plugin::ExportsPlugin;
use crate::plugins::jobs::plugin::JobsPlugin;
use crate::plugins::spawn::plugin::SpawnPlugin;
use crate::plugins::system::plugin::SystemPlugin;
use crate::plugins::translations::plugin::TranslationsPlugin;
use crate::plugins::visuals::plugin::VisualsPlugin;

/// One domain's mirrored surface: its typed Specta builder and the raw commands that builder cannot hold.
///
/// Raw commands travel beside the builder rather than in a second list, because a forgotten raw export fails silently
/// - no error, just a binding file that is never written - and only surfaces as a missing import in the frontend.
struct CommandModule<R: tauri::Runtime> {
  name: &'static str,
  builder: tauri_specta::Builder<R>,
  /// Raw commands of the domain, empty for a domain declaring no `@raw` block.
  raw: &'static [(&'static str, &'static [(&'static str, &'static str)])],
}

/// Every Tauri plugin whose commands are mirrored.
///
/// A domain with no typed command at all would be absent: there would be no typed module to write.
fn command_modules<R: tauri::Runtime>() -> Vec<CommandModule<R>> {
  vec![
    CommandModule {
      name: AssetsPlugin::NAME,
      builder: AssetsPlugin::specta_builder::<R>(),
      raw: crate::ipc::registry::assets::RAW_COMMANDS,
    },
    CommandModule {
      name: ArchivesPlugin::NAME,
      builder: ArchivesPlugin::specta_builder::<R>(),
      raw: crate::ipc::registry::archives::RAW_COMMANDS,
    },
    CommandModule {
      name: ConfigsPlugin::NAME,
      builder: ConfigsPlugin::specta_builder::<R>(),
      raw: crate::ipc::registry::configs::RAW_COMMANDS,
    },
    CommandModule {
      name: DialogsPlugin::NAME,
      builder: DialogsPlugin::specta_builder::<R>(),
      raw: crate::ipc::registry::dialogs::RAW_COMMANDS,
    },
    CommandModule {
      name: EquipmentIconsPlugin::NAME,
      builder: EquipmentIconsPlugin::specta_builder::<R>(),
      raw: crate::ipc::registry::equipment_icons::RAW_COMMANDS,
    },
    CommandModule {
      name: ExportsPlugin::NAME,
      builder: ExportsPlugin::specta_builder::<R>(),
      raw: crate::ipc::registry::exports::RAW_COMMANDS,
    },
    CommandModule {
      name: JobsPlugin::NAME,
      builder: JobsPlugin::specta_builder::<R>(),
      raw: crate::ipc::registry::jobs::RAW_COMMANDS,
    },
    CommandModule {
      name: SpawnPlugin::NAME,
      builder: SpawnPlugin::specta_builder::<R>(),
      raw: crate::ipc::registry::spawn::RAW_COMMANDS,
    },
    CommandModule {
      name: SystemPlugin::NAME,
      builder: SystemPlugin::specta_builder::<R>(),
      raw: crate::ipc::registry::system::RAW_COMMANDS,
    },
    CommandModule {
      name: TranslationsPlugin::NAME,
      builder: TranslationsPlugin::specta_builder::<R>(),
      raw: crate::ipc::registry::translations::RAW_COMMANDS,
    },
    CommandModule {
      name: VisualsPlugin::NAME,
      builder: VisualsPlugin::specta_builder::<R>(),
      raw: crate::ipc::registry::visuals::RAW_COMMANDS,
    },
  ]
}

/// Path of the mirrors the frontend compiles against.
fn committed_bindings() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../xrf-ui/src/core/bindings")
}

/// Rewrite the committed mirrors in place.
///
/// Ignored so that a plain workspace test run cannot dirty the tree: the output is formatted by the frontend
/// toolchain afterwards, which only `cargo make generate-typescript` goes on to do.
#[test]
#[ignore]
fn export_typescript_bindings() {
  export_bindings_to(&committed_bindings());
}

/// Fail only where the committed mirrors no longer describe what the Rust sources produce.
#[test]
fn verify_typescript_bindings() {
  let generated: PathBuf = build_absolute_generated_test_resource_path("bindings");

  export_bindings_to(&generated);

  let drift: SurfaceDrift = compare_surfaces(&read_surface(&committed_bindings()), &read_surface(&generated));

  assert!(!drift.is_breaking(), "{}", drift.describe());
}

fn export_bindings_to(output: &Path) {
  let types_output: PathBuf = output.join(TYPES_DIRECTORY);
  let commands_output: PathBuf = output.join(COMMANDS_DIRECTORY);

  reset_directory(&types_output);
  reset_directory(&commands_output);

  let collected: Arc<Mutex<Types>> = Arc::new(Mutex::new(Types::default()));
  let modules: Vec<CommandModule<tauri::Wry>> = command_modules();

  for module in &modules {
    module
      .builder
      .export(
        command_exporter(Arc::clone(&collected)),
        commands_output.join(format!("{}.ts", module.name)),
      )
      .unwrap_or_else(|error| panic!("Failed to export {} commands: {error}", module.name));
  }

  let collected: Types = Arc::try_unwrap(collected)
    .unwrap_or_else(|_| panic!("Collected types are still borrowed"))
    .into_inner()
    .expect("Collected types lock is poisoned");
  let ownership: TypeOwnership = export_type_modules(&types_output, &collected);

  for module in &modules {
    finalize_command_module(
      &commands_output.join(format!("{}.ts", module.name)),
      module.name,
      &ownership,
    );

    if !module.raw.is_empty() {
      export_raw_commands(
        &commands_output.join(format!("{}-raw.ts", module.name)),
        module.name,
        module.raw,
        &ownership,
      );
    }
  }
}
