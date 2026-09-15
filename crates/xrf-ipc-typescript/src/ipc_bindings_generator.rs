use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use specta::Types;
use tauri::Runtime;

use crate::bindings_surface::{Surface, SurfaceDrift};
use crate::command_module_writer::CommandModuleWriter;
use crate::command_typescript::CommandTypescript;
use crate::constants::{COMMANDS_DIRECTORY, TYPES_DIRECTORY};
use crate::enumerations::Enumerations;
use crate::generated_output::reset_directory;
use crate::ipc_command_surface::IpcCommandSurface;
use crate::type_module_writer::TypeModuleWriter;
use crate::type_ownership::TypeOwnership;

/// Writes the frontend's TypeScript mirror of an IPC command surface.
///
/// The order below is the whole contract and it is not a caller's to reproduce: every plugin's commands are exported
/// first, because that is what collects the types they reference; the type modules are written next, because only then
/// is it known which module declares what; and each command module is finalized last, because it is the type modules
/// that say which imports replace the copies Specta inlined.
pub struct IpcBindingsGenerator;

impl IpcBindingsGenerator {
  /// Writes the mirror of `surfaces` into `output`, replacing the `types/` and `commands/` it already held.
  ///
  /// # Panics
  ///
  /// Panics when the output cannot be written, or when a generated module no longer satisfies an invariant the
  /// mirrors rest on — a type owned by two modules, a type module importing another, a foreign declaration with
  /// nowhere to live, or an `invoke` that would bypass the counted wrapper.
  pub fn generate<R: Runtime>(output: &Path, surfaces: &[IpcCommandSurface<R>]) {
    let types: PathBuf = output.join(TYPES_DIRECTORY);
    let commands: PathBuf = output.join(COMMANDS_DIRECTORY);

    reset_directory(&types);
    reset_directory(&commands);

    let collected: Types = Self::export_commands(&commands, surfaces);
    let (ownership, enumerations): (TypeOwnership, Enumerations) = TypeModuleWriter::write_all(&types, &collected);

    for surface in surfaces {
      CommandModuleWriter::finalize(
        &Self::module_path(&commands, surface.name),
        surface.name,
        &ownership,
        &enumerations,
      );

      if !surface.raw.is_empty() {
        CommandModuleWriter::write_raw(
          &commands.join(format!("{}-raw.ts", surface.name)),
          surface.name,
          surface.raw,
          &ownership,
          &enumerations,
        );
      }
    }
  }

  /// Generates into `scratch` and reports how the mirrors committed at `committed` differ from it.
  ///
  /// # Panics
  ///
  /// Panics on anything [`Self::generate`] panics on, and when either tree cannot be read back as TypeScript.
  pub fn verify<R: Runtime>(committed: &Path, scratch: &Path, surfaces: &[IpcCommandSurface<R>]) -> SurfaceDrift {
    Self::generate(scratch, surfaces);

    Surface::read(committed).compare_to(&Surface::read(scratch))
  }

  /// Exports every plugin's commands, answering with the types those commands referenced.
  ///
  /// Specta inlines the full transitive closure of a plugin's signatures and cannot reference a declaration living in
  /// another file, so the closure is collected here to be written once and imported afterwards.
  fn export_commands<R: Runtime>(commands: &Path, surfaces: &[IpcCommandSurface<R>]) -> Types {
    let collected: Arc<Mutex<Types>> = Arc::new(Mutex::new(Types::default()));

    for surface in surfaces {
      surface
        .builder
        .export(
          CommandTypescript::collecting(Arc::clone(&collected)),
          Self::module_path(commands, surface.name),
        )
        .unwrap_or_else(|error| panic!("Failed to export {} commands: {error}", surface.name));
    }

    Arc::try_unwrap(collected)
      .unwrap_or_else(|_| panic!("An exporter still holds the collected types"))
      .into_inner()
      .expect("Collected types lock is poisoned")
  }

  /// Where one plugin's command module is written, which is also where it is read back to be finalized.
  fn module_path(commands: &Path, plugin: &str) -> PathBuf {
    commands.join(format!("{plugin}.ts"))
  }
}
