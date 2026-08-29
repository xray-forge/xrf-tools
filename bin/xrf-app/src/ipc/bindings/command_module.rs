//! Writing `core/bindings/commands/`, one module per Tauri plugin.

use std::fs;
use std::path::Path;

use xrf_utils::{format_path, to_camel_case};

use crate::ipc::bindings::constants::{GENERATED_HEADER, TYPES_MARKER};
use crate::ipc::bindings::output::write_generated;
use crate::ipc::bindings::ownership::TypeOwnership;

/// Replaces the types Tauri Specta inlined into a command module with imports, and names its commands.
///
/// The generated symbol was `commands` in every module, which left nine identical names in the directory and
/// made every call site alias it by hand.
pub(super) fn finalize_command_module(path: &Path, plugin: &str, ownership: &TypeOwnership) {
  let contents: String =
    fs::read_to_string(path).unwrap_or_else(|error| panic!("Failed to read {}: {error}", format_path(path)));
  let commands: String = contents
    .split_once(TYPES_MARKER)
    .map_or(contents.as_str(), |(before, _)| before)
    .trim_end()
    .replace(
      "export const commands = {",
      &format!("export const {}Commands = {{", to_camel_case(plugin)),
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
pub(super) fn export_raw_commands(
  path: &Path,
  plugin: &str,
  commands: &[(&str, &[(&str, &str)])],
  ownership: &TypeOwnership,
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
