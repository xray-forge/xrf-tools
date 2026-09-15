//! Writing `core/ipc/commands/`, one module per Tauri plugin.

use std::fs;
use std::path::Path;

use xrf_utils::{format_path, to_camel_case};

use crate::constants::GENERATED_HEADER;
use crate::enumerations::Enumerations;
use crate::generated_output::write_generated;
use crate::type_ownership::TypeOwnership;

/// Writes `commands/`, one module per Tauri plugin.
pub(crate) struct CommandModuleWriter;

impl CommandModuleWriter {
  /// Marker Tauri Specta writes before the types it inlines into a command module.
  ///
  /// Everything after it is a copy of a declaration owned by the crate the type comes from, so it is the cut point
  /// where those copies are dropped in favour of imports.
  const TYPES_MARKER: &'static str = "/* Types */";

  /// Tauri Specta's own invoke import, for a module that also carries a job command's channel.
  const SPECTA_INVOKE_IMPORT_WITH_CHANNEL: &'static str =
    "import { invoke as __TAURI_INVOKE, Channel } from \"@tauri-apps/api/core\";";

  /// The same import for a module with no channel, which is the only other spelling Specta writes.
  const SPECTA_INVOKE_IMPORT: &'static str = "import { invoke as __TAURI_INVOKE } from \"@tauri-apps/api/core\";";

  /// Where a generated module takes `invoke` from instead.
  ///
  /// The wrapper counts every call, which is why the redirect is written here rather than left to a hundred call
  /// sites to remember: a command added later is measured by existing. `Channel` still comes from Tauri, being a
  /// transport handle the wrapper has no business in.
  const COUNTED_INVOKE_IMPORT: &'static str = "import { invoke as __TAURI_INVOKE } from \"@/core/ipc/invoke\";";

  /// The channel-carrying spelling, split so the redirect leaves `Channel` where it was.
  const CHANNEL_IMPORT_AND_COUNTED_INVOKE: &'static str = "import { Channel } from \"@tauri-apps/api/core\";
import { invoke as __TAURI_INVOKE } from \"@/core/ipc/invoke\";";

  /// Replaces the types Tauri Specta inlined into a command module with imports, and names its commands.
  ///
  /// The generated symbol was `commands` in every module, which left nine identical names in the directory and
  /// made every call site alias it by hand.
  pub(crate) fn finalize(path: &Path, plugin: &str, ownership: &TypeOwnership, enumerations: &Enumerations) {
    let contents: String =
      fs::read_to_string(path).unwrap_or_else(|error| panic!("Failed to read {}: {error}", format_path(path)));
    let commands: String = contents
      .split_once(Self::TYPES_MARKER)
      .map_or(contents.as_str(), |(before, _)| before)
      .trim_end()
      .replace(
        "export const commands = {",
        &format!("export const {}Commands = {{", to_camel_case(plugin)),
      )
      // The wider spelling first: replacing the bare one first would leave its `, Channel` behind.
      .replace(
        Self::SPECTA_INVOKE_IMPORT_WITH_CHANNEL,
        Self::CHANNEL_IMPORT_AND_COUNTED_INVOKE,
      )
      .replace(Self::SPECTA_INVOKE_IMPORT, Self::COUNTED_INVOKE_IMPORT);

    let commands: String = enumerations.rewrite_parameters(&commands);

    // A module still importing `invoke` from Tauri would be dispatched and never counted, and nothing downstream would
    // say so. Specta changing how it writes that import is the way this stops being true, so it is asserted here.
    assert!(
      commands.contains(Self::COUNTED_INVOKE_IMPORT),
      "{} does not route invoke through the counted wrapper; Tauri Specta's import spelling has changed",
      format_path(path)
    );

    ownership.assert_no_foreign_references(&commands, plugin);

    let (header, body) = commands
      .split_once("\n\n")
      .unwrap_or_else(|| panic!("{} has no header", format_path(path)));

    write_generated(
      path,
      &format!(
        "{header}\n\n{}{}\n",
        ownership.imports_for(&commands),
        body.trim_start()
      ),
    );
  }

  /// Writes the wrappers for one domain's raw commands.
  ///
  /// Specta cannot collect a command returning `tauri::ipc::Response`, so these are generated from the registry
  /// instead of written by hand. The registry carries each argument's TypeScript type for exactly this reason;
  /// the return is always `ArrayBuffer`, which is what the raw response arrives as.
  pub(crate) fn write_raw(
    path: &Path,
    plugin: &str,
    commands: &[(&str, &[(&str, &str)])],
    ownership: &TypeOwnership,
    enumerations: &Enumerations,
  ) {
    if commands.is_empty() {
      return;
    }

    let mut wrappers: String = String::new();

    for (wire_name, arguments) in commands {
      let parameters: String = arguments
        .iter()
        .map(|(name, argument_type)| format!("{name}: {argument_type}"))
        .collect::<Vec<String>>()
        .join(", ");
      let payload: String = arguments
        .iter()
        .map(|(name, _)| (*name).to_string())
        .collect::<Vec<String>>()
        .join(", ");

      wrappers.push_str(&format!(
        "  {}: ({parameters}): Promise<ArrayBuffer> =>\n    invokeRaw(\"plugin:{plugin}|{wire_name}\", {{ {payload} }}),\n",
        to_camel_case(wire_name)
      ));
    }

    // The registry spells these argument types by hand, so they take the same retyping as a Specta signature.
    let wrappers: String = enumerations.rewrite_parameters(&wrappers);

    ownership.assert_no_foreign_references(&wrappers, plugin);

    write_generated(
      path,
      &format!(
        "{GENERATED_HEADER}\nimport {{ invokeRaw }} from \"@/core/ipc/raw\";\n{}\n/** Commands answering with raw bytes, which Specta cannot type. */\nexport const {}RawCommands = {{\n{wrappers}}};\n",
        ownership.imports_for(&wrappers),
        to_camel_case(plugin)
      ),
    );
  }
}
