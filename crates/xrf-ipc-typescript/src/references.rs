//! Reading generated TypeScript back, to learn which declarations a block of it depends on.
//!
//! Specta renders each module in isolation, so the imports tying the modules together have to be recovered
//! from the rendered text rather than from the type graph.

use std::collections::{BTreeMap, BTreeSet};

use crate::constants::{BINDINGS_ROOT, TYPES_DIRECTORY};

/// Generated source with doc comments and string literals blanked out, byte for byte.
///
/// A type reference cannot be told from surrounding text by shape alone: class ids and pack modes render as
/// PascalCase string unions, and doc comment prose is capitalised. Removing both leaves only real code.
///
/// Every blanked character is replaced by as many spaces as it occupied, so a position in the result names
/// the same position in the source. That is what lets a rewrite splice into the original text at offsets
/// found here, rather than having to find them again in a copy that has already drifted.
fn without_comments_and_strings(source: &str) -> String {
  /// A blanked run keeps its newlines, so a line number computed either side of this stays the same.
  fn blank(stripped: &mut String, character: char) {
    match character {
      '\n' => stripped.push('\n'),
      other => stripped.extend(std::iter::repeat_n(' ', other.len_utf8())),
    }
  }

  let mut stripped: String = String::with_capacity(source.len());
  let mut characters = source.chars().peekable();

  while let Some(character) = characters.next() {
    match character {
      '/' if characters.peek() == Some(&'/') => {
        stripped.push_str("  ");
        characters.next();

        for skipped in characters.by_ref() {
          blank(&mut stripped, skipped);

          if skipped == '\n' {
            break;
          }
        }
      }
      '/' if characters.peek() == Some(&'*') => {
        stripped.push_str("  ");
        characters.next();

        let mut previous: char = '\0';

        for skipped in characters.by_ref() {
          blank(&mut stripped, skipped);

          if previous == '*' && skipped == '/' {
            break;
          }

          previous = skipped;
        }
      }
      '"' | '\'' | '`' => {
        stripped.push(' ');

        for skipped in characters.by_ref() {
          blank(&mut stripped, skipped);

          if skipped == character {
            break;
          }
        }
      }
      _ => stripped.push(character),
    }
  }

  stripped
}

/// Calls `visit` with the byte range of every identifier in `blanked` that is not a property name.
///
/// Both readers of the scan share it, so the set of names an import is written for and the set a rewrite
/// replaces can never disagree about what counts as a reference. Takes the blanked text rather than the
/// source, because a caller narrowing the scan by position has to blank it to find those positions anyway.
fn for_each_identifier(blanked: &str, mut visit: impl FnMut(usize, usize, &str)) {
  let bytes: &[u8] = blanked.as_bytes();
  let mut index: usize = 0;

  while index < bytes.len() {
    if !bytes[index].is_ascii_alphabetic() && bytes[index] != b'_' {
      index += 1;
      continue;
    }

    let start: usize = index;

    while index < bytes.len() && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_') {
      index += 1;
    }

    // A property name is never a type reference; `Shape` renders its variants as `{ Sphere: .. }`.
    let trailing: &str = blanked[index..].trim_start();

    if trailing.starts_with(':') || trailing.starts_with("?:") {
      continue;
    }

    visit(start, index, &blanked[start..index]);
  }
}

/// Names of generated types that `source` references and `owner` does not itself declare.
pub(crate) fn referenced_types<'a>(
  source: &str,
  owners: &'a BTreeMap<String, String>,
  owner: &str,
) -> BTreeSet<&'a str> {
  let blanked: String = without_comments_and_strings(source);
  let mut referenced: BTreeSet<&str> = BTreeSet::new();

  for_each_identifier(&blanked, |_, _, identifier| {
    if let Some((name, module)) = owners.get_key_value(identifier)
      && module != owner
    {
      referenced.insert(name.as_str());
    }
  });

  referenced
}

/// `source` with every type reference `renames` covers replaced by the name it maps to, in parameters only.
///
/// Replacing text rather than rewriting a parsed module: the generated modules carry the Rust
/// doc comments, which SWC keeps outside the tree and a re-render would drop. This reuses the same identifier
/// scan the imports are computed from, so a reference is rewritten exactly when it would have been imported,
/// and prose naming a type in a doc comment is left alone.
///
/// A return is left alone because it is not something a caller writes: what arrives is a raw string off the IPC
/// channel, and spelling it as a nominal enum asserts that the backend sent a declared member rather than
/// checking it. The two answers differ — `toEnumMember` throws on an undeclared spelling, an enum return throws
/// nowhere — and the frontend cannot tell the difference by reading the type. The union says what it is.
pub(crate) fn rewrite_parameter_type_references(source: &str, renames: &BTreeMap<String, String>) -> String {
  let blanked: String = without_comments_and_strings(source);
  let returns: Vec<(usize, usize)> = invoke_return_ranges(&blanked);
  let mut rewritten: String = String::with_capacity(source.len());
  let mut copied: usize = 0;

  for_each_identifier(&blanked, |start, end, identifier| {
    if returns.iter().any(|(from, to)| start >= *from && start < *to) {
      return;
    }

    if let Some(replacement) = renames.get(identifier) {
      rewritten.push_str(&source[copied..start]);
      rewritten.push_str(replacement);
      copied = end;
    }
  });

  rewritten.push_str(&source[copied..]);
  rewritten
}

/// Byte ranges of `blanked` that hold a command's return type.
///
/// Tauri Specta writes a command body as `__TAURI_INVOKE<Return>("plugin:..", { .. })` and annotates no return
/// on the arrow, so the sole type argument of that call is the whole of the return surface. The raw wrappers
/// this is also applied to call `invokeRaw` and annotate `Promise<ArrayBuffer>`, which names no enum; nothing is
/// excluded there and nothing needs to be.
fn invoke_return_ranges(blanked: &str) -> Vec<(usize, usize)> {
  const INVOKE: &str = "__TAURI_INVOKE";

  let mut ranges: Vec<(usize, usize)> = Vec::new();

  for (call, _) in blanked.match_indices(INVOKE) {
    let after: usize = call + INVOKE.len();
    let arguments: usize = after + blanked[after..].len() - blanked[after..].trim_start().len();

    // The import binding this name, which is the one occurrence carrying no type argument.
    if !blanked[arguments..].starts_with('<') {
      continue;
    }

    let mut depth: usize = 0;
    let end: usize = blanked[arguments..]
      .char_indices()
      .find_map(|(offset, character)| match character {
        '<' => {
          depth += 1;
          None
        }
        '>' => {
          depth -= 1;
          (depth == 0).then_some(arguments + offset)
        }
        _ => None,
      })
      .unwrap_or_else(|| panic!("`{INVOKE}` at byte {call} opens a type argument that is never closed"));

    ranges.push((arguments + 1, end));
  }

  ranges
}

/// Import statements pulling every referenced type from the module that declares it.
pub(crate) fn render_imports(referenced: &BTreeSet<&str>, owners: &BTreeMap<String, String>) -> String {
  let mut grouped: BTreeMap<&str, Vec<&str>> = BTreeMap::new();

  for name in referenced {
    grouped
      .entry(
        owners
          .get(*name)
          .unwrap_or_else(|| panic!("`{name}` is referenced but has no owning module"))
          .as_str(),
      )
      .or_default()
      .push(name);
  }

  grouped
    .into_iter()
    .map(|(module, names)| {
      format!(
        "import {{ {} }} from \"{BINDINGS_ROOT}/{TYPES_DIRECTORY}/{module}\";\n",
        names.join(", ")
      )
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use std::collections::BTreeMap;

  use super::rewrite_parameter_type_references;

  fn renames() -> BTreeMap<String, String> {
    BTreeMap::from([
      ("XrayAssetType".to_string(), "EXrayAssetType".to_string()),
      ("DialogProjectMode".to_string(), "EDialogProjectMode".to_string()),
    ])
  }

  #[test]
  fn a_parameter_is_rewritten_to_the_enum() {
    assert_eq!(
      rewrite_parameter_type_references(
        "  listAssets: (roots: XrayRoots, kind: XrayAssetType) =>\n    __TAURI_INVOKE<Array<XrayAsset>>(\"plugin:assets|list_assets\", { roots, kind }),\n",
        &renames()
      ),
      "  listAssets: (roots: XrayRoots, kind: EXrayAssetType) =>\n    __TAURI_INVOKE<Array<XrayAsset>>(\"plugin:assets|list_assets\", { roots, kind }),\n"
    );
  }

  #[test]
  fn a_return_keeps_the_union_it_is_read_back_as() {
    let source: &str = "  detectMode: (roots: XrayRoots) => __TAURI_INVOKE<DialogProjectMode>(\"plugin:dialogs|detect_mode\", { roots }),\n";

    assert_eq!(rewrite_parameter_type_references(source, &renames()), source);
  }

  #[test]
  fn a_nested_return_type_is_excluded_to_its_closing_bracket() {
    // Depth-counted rather than read to the first `>`, or everything after `SessionSnapshot<` would be rewritten
    // again — including the arguments of the next command in the module.
    let source: &str = concat!(
      "  getProject: () => __TAURI_INVOKE<SessionRestore<Array<DialogProjectMode>>>(\"plugin:dialogs|get_project\"),\n",
      "  openProject: (mode: DialogProjectMode) => __TAURI_INVOKE<null>(\"plugin:dialogs|open_project\", { mode }),\n",
    );

    assert_eq!(
      rewrite_parameter_type_references(source, &renames()),
      concat!(
        "  getProject: () => __TAURI_INVOKE<SessionRestore<Array<DialogProjectMode>>>(\"plugin:dialogs|get_project\"),\n",
        "  openProject: (mode: EDialogProjectMode) => __TAURI_INVOKE<null>(\"plugin:dialogs|open_project\", { mode }),\n",
      )
    );
  }

  #[test]
  fn the_import_binding_the_invoke_name_opens_no_return() {
    // `import { invoke as __TAURI_INVOKE }` is the one occurrence with no type argument after it. Treating it as one
    // would exclude the rest of the module from the rewrite.
    let source: &str = concat!(
      "import { invoke as __TAURI_INVOKE } from \"@/core/ipc/invoke\";\n",
      "  listAssets: (kind: XrayAssetType) => __TAURI_INVOKE<null>(\"plugin:assets|list_assets\", { kind }),\n",
    );

    assert_eq!(
      rewrite_parameter_type_references(source, &renames()),
      concat!(
        "import { invoke as __TAURI_INVOKE } from \"@/core/ipc/invoke\";\n",
        "  listAssets: (kind: EXrayAssetType) => __TAURI_INVOKE<null>(\"plugin:assets|list_assets\", { kind }),\n",
      )
    );
  }

  #[test]
  fn a_raw_wrapper_has_no_invoke_call_and_is_rewritten_whole() {
    // `Promise<ArrayBuffer>` is every raw wrapper's return and names no enum, so nothing there needs excluding.
    assert_eq!(
      rewrite_parameter_type_references(
        "  readCandidate: (kind: XrayAssetType): Promise<ArrayBuffer> =>\n    invokeRaw(\"plugin:textures|read_candidate\", { kind }),\n",
        &renames()
      ),
      "  readCandidate: (kind: EXrayAssetType): Promise<ArrayBuffer> =>\n    invokeRaw(\"plugin:textures|read_candidate\", { kind }),\n"
    );
  }

  #[test]
  fn prose_naming_a_type_in_a_doc_comment_is_left_alone() {
    let source: &str = "  /** Answers a DialogProjectMode. */\n  detectMode: (mode: DialogProjectMode) => null,\n";

    assert_eq!(
      rewrite_parameter_type_references(source, &renames()),
      "  /** Answers a DialogProjectMode. */\n  detectMode: (mode: EDialogProjectMode) => null,\n"
    );
  }
}
