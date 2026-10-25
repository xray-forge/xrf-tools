use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use tauri::Runtime;

use crate::collected::{CollectedCommands, CollectedConstants, CommandTypescript};
use crate::enumerations::Enumerations;
use crate::input::{IpcCommandSurface, MirrorImports, TypeModules};
use crate::verify::{Surface, SurfaceDrift};
use crate::write::{CommandModuleWriter, GeneratedOutput, PluginModules, TypeModuleWriter, TypeOwnership};

/// Writes the frontend's TypeScript mirror of an IPC command surface.
pub struct IpcBindingsGenerator;

impl IpcBindingsGenerator {
  /// Name of the module merging every plugin's signatures, beside the command modules.
  const SIGNATURES_MODULE: &'static str = "signatures.ts";

  /// Writes the mirror of `surfaces` into `output`, replacing the `types/` and `commands/` it already held, each type
  /// in the module `type_modules` gives its crate, every module addressing the others and the runtime as `imports`
  /// says.
  ///
  /// # Panics
  ///
  /// Panics when the output cannot be written, or when a generated module no longer satisfies an invariant the
  /// mirrors rest on — a type owned by two modules, a type module importing another, a foreign declaration with
  /// nowhere to live, a wrapper Specta spelled differently, or an `invoke` that would bypass the counted wrapper.
  pub fn generate<R: Runtime>(
    output: &Path,
    surfaces: &[IpcCommandSurface<R>],
    type_modules: &TypeModules,
    imports: &MirrorImports,
  ) {
    Self::assert_distinct_names(surfaces);

    let types: PathBuf = output.join(GeneratedOutput::TYPES_DIRECTORY);
    let commands: PathBuf = output.join(GeneratedOutput::COMMANDS_DIRECTORY);

    GeneratedOutput::reset_directory(&types);
    GeneratedOutput::reset_directory(&commands);

    let mut collected: CollectedCommands = Self::export_commands(&commands, surfaces);
    let constants: CollectedConstants = CollectedConstants::collect(type_modules, &mut collected.types);
    let (ownership, enumerations): (TypeOwnership, Enumerations) =
      TypeModuleWriter::write_all(&types, &collected.types, &constants, type_modules, imports);
    let mut plugins: Vec<PluginModules> = Vec::with_capacity(surfaces.len());

    for surface in surfaces {
      CommandModuleWriter::finalize(
        &Self::module_path(&commands, surface.name),
        surface.name,
        collected.commands_of(surface.name),
        &ownership,
        &enumerations,
        imports,
      );

      if !surface.raw.is_empty() {
        CommandModuleWriter::write_raw(
          &commands.join(format!("{}-raw.ts", surface.name)),
          surface.name,
          surface.raw,
          &ownership,
          &enumerations,
          imports,
        );
      }

      if !surface.bulk.is_empty() {
        CommandModuleWriter::write_bulk(
          &commands.join(format!("{}-bulk.ts", surface.name)),
          surface.name,
          surface.bulk,
          &ownership,
          &enumerations,
          imports,
        );
      }

      plugins.push(PluginModules {
        name: surface.name,
        has_raw: !surface.raw.is_empty(),
        has_bulk: !surface.bulk.is_empty(),
      });
    }

    CommandModuleWriter::write_signatures(&commands.join(Self::SIGNATURES_MODULE), &plugins, imports);
  }

  /// Generates into `scratch` and reports how the mirrors committed at `committed` differ from it.
  ///
  /// # Panics
  ///
  /// Panics on anything [`Self::generate`] panics on, and when either tree cannot be read back as TypeScript.
  pub fn verify<R: Runtime>(
    committed: &Path,
    scratch: &Path,
    surfaces: &[IpcCommandSurface<R>],
    type_modules: &TypeModules,
    imports: &MirrorImports,
  ) -> SurfaceDrift {
    Self::generate(scratch, surfaces, type_modules, imports);

    Surface::read(committed).compare_to(&Surface::read(scratch))
  }

  /// Exports every plugin's commands, answering with the commands and the types those commands referenced.
  ///
  /// Specta inlines the full transitive closure of a plugin's signatures and cannot reference a declaration living in
  /// another file, so the closure is collected here to be written once and imported afterwards.
  fn export_commands<R: Runtime>(commands: &Path, surfaces: &[IpcCommandSurface<R>]) -> CollectedCommands {
    let collected: Arc<Mutex<CollectedCommands>> = Arc::new(Mutex::new(CollectedCommands::default()));

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
      .unwrap_or_else(|_| panic!("An exporter still holds the collected commands"))
      .into_inner()
      .expect("Collected commands lock is poisoned")
  }

  /// Fails when two surfaces carry one name: both would write one command module, and the commands exported second
  /// would be the only ones collected for it.
  fn assert_distinct_names<R: Runtime>(surfaces: &[IpcCommandSurface<R>]) {
    let mut names: BTreeSet<&str> = BTreeSet::new();

    for surface in surfaces {
      assert!(
        names.insert(surface.name),
        "Two command surfaces are named `{}`",
        surface.name
      );
    }
  }

  /// Where one plugin's command module is written, which is also where it is read back to be finalized.
  fn module_path(commands: &Path, plugin: &str) -> PathBuf {
    commands.join(format!("{plugin}.ts"))
  }
}
