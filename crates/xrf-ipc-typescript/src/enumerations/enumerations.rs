use std::collections::BTreeMap;
use std::panic::Location;

use specta::Types;
use specta::datatype::NamedDataType;

use crate::enumerations::enumeration::Enumeration;
use crate::enumerations::enumeration_member::EnumerationMember;
use crate::enumerations::enumeration_subject::EnumerationSubject;
use crate::typescript::source::{for_each_identifier, without_comments_and_strings};

/// Every exported type that declares a TypeScript enum, keyed by the type's own name.
pub(crate) struct Enumerations {
  declared: BTreeMap<String, Enumeration>,
}

impl Enumerations {
  /// Resolves which collected types declare an enum, pairing the Rust variant names with the wire ones.
  ///
  /// Two views of one type are read together: `types` has been through the TypeScript format and carries the
  /// serialized spellings, while `collected` is the Rust source and carries the identifiers a member is named from.
  ///
  /// # Panics
  ///
  /// Panics when a mapped type has no Rust source, which means the two views can no longer be related at all.
  pub(crate) fn resolve(collected: &Types, types: &Types) -> Self {
    let sources: BTreeMap<(&str, Location<'static>), &NamedDataType> = collected
      .into_sorted_iter()
      .map(|named| ((named.module_path.as_ref(), named.location), named))
      .collect();
    let mut declared: BTreeMap<String, Enumeration> = BTreeMap::new();

    for named in types.into_sorted_iter() {
      let Some((subject, spellings)) = EnumerationSubject::of(named) else {
        continue;
      };

      // Located by declaration site rather than by name: a rename leaves the two views agreeing on nothing else.
      let source: &NamedDataType = sources
        .get(&(named.module_path.as_ref(), named.location))
        .unwrap_or_else(|| panic!("`{}` was mapped from no collected type", named.name));
      let members: Vec<EnumerationMember> = EnumerationMember::pair(named, source, spellings);

      EnumerationMember::assert_distinct(&named.name, &members);
      declared.insert(named.name.to_string(), Enumeration { subject, members });
    }

    Self { declared }
  }

  /// The enum `name` declares, or `None` for a type that declares none.
  pub(crate) fn enum_name_of(&self, name: &str) -> Option<String> {
    self
      .declared
      .contains_key(name)
      .then(|| Enumeration::to_enum_name(name))
  }

  /// Every type name whose enum may stand in for the type itself, mapped to it.
  ///
  /// A discriminant enum names one field of a union, not the union, so rewriting a parameter to it would ask a caller
  /// to pass `EArchiveSubject.WORLD` where a whole `ArchiveSubject` is wanted.
  ///
  /// What a command *parameter* is rewritten to, and nothing else. A parameter naming the union would accept a
  /// quoted literal the Rust side never declared, and there the enum costs nothing: the caller is writing the
  /// value, so it can name the member. A return is the opposite — a raw runtime string off the IPC channel,
  /// which an enum would assert about rather than check — so
  /// [`Self::rewrite_parameters`]
  /// leaves returns spelled as the union.
  ///
  /// Struct fields in `types/` are deliberately **not** rewritten, which leaves 54 of them spelled as the union. That
  /// costs less than it looks: an enum member is assignable to the literal it equals, so
  /// `{ mode: EArchivePackMode.COMPRESS }` already compiles wherever `ArchivePackConfig` is built by hand, and only
  /// the *enforcement* is missing. Enforcing it is not closable here: a field has no direction of its own, only the
  /// struct does, and a struct can have both. `ArchivePackConfig` is returned by `default_pack_config` and taken by `pack` and `list_pack_volumes`
  /// — the frontend asks the backend for one, edits it in a form and sends it back — so whichever of the two
  /// spellings its `mode` were given would be wrong for the other half of that round trip. Closing it needs a
  /// per-direction type, which is a Rust-side split of a type the Rust side has one of, not a generator change.
  fn references(&self) -> BTreeMap<String, String> {
    self
      .declared
      .iter()
      .filter(|(_, enumeration)| enumeration.is_type_itself())
      .map(|(name, _)| (name.clone(), Enumeration::to_enum_name(name)))
      .collect()
  }

  /// One type as the frontend declares it: whatever Specta rendered, with the enum this type declares.
  ///
  /// The rendered union is taken rather than produced so that this one call answers for every type, enum or not, and
  /// a caller never has to know which shape it is holding.
  pub(crate) fn render(&self, named: &NamedDataType, union: &str) -> String {
    self
      .declared
      .get(named.name.as_ref())
      .map_or_else(|| union.to_owned(), |enumeration| enumeration.render(named, union))
  }

  /// Scan the imports are computed from, so a reference is rewritten exactly when it would have been imported,
  /// and prose naming a type in a doc comment is left alone.
  pub(crate) fn rewrite_parameters(&self, source: &str) -> String {
    Self::rewrite_parameters_in(source, &self.references())
  }

  /// The rewrite itself, which the renames are handed to rather than looked up.
  fn rewrite_parameters_in(source: &str, renames: &BTreeMap<String, String>) -> String {
    let blanked: String = without_comments_and_strings(source);
    let returns: Vec<(usize, usize)> = Self::invoke_return_ranges(&blanked);
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
}

#[cfg(test)]
mod tests {
  use std::collections::BTreeMap;

  use super::Enumerations;

  fn renames() -> BTreeMap<String, String> {
    BTreeMap::from([
      ("XrayAssetType".to_string(), "EXrayAssetType".to_string()),
      ("DialogProjectMode".to_string(), "EDialogProjectMode".to_string()),
    ])
  }

  #[test]
  fn a_parameter_is_rewritten_to_the_enum() {
    assert_eq!(
      Enumerations::rewrite_parameters_in(
        "  listAssets: (roots: XrayRoots, kind: XrayAssetType) =>\n    __TAURI_INVOKE<Array<XrayAsset>>(\"plugin:assets|list_assets\", { roots, kind }),\n",
        &renames()
      ),
      "  listAssets: (roots: XrayRoots, kind: EXrayAssetType) =>\n    __TAURI_INVOKE<Array<XrayAsset>>(\"plugin:assets|list_assets\", { roots, kind }),\n"
    );
  }

  #[test]
  fn a_return_keeps_the_union_it_is_read_back_as() {
    let source: &str = "  detectMode: (roots: XrayRoots) => __TAURI_INVOKE<DialogProjectMode>(\"plugin:dialogs|detect_mode\", { roots }),\n";

    assert_eq!(Enumerations::rewrite_parameters_in(source, &renames()), source);
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
      Enumerations::rewrite_parameters_in(source, &renames()),
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
      Enumerations::rewrite_parameters_in(source, &renames()),
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
      Enumerations::rewrite_parameters_in(
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
      Enumerations::rewrite_parameters_in(source, &renames()),
      "  /** Answers a DialogProjectMode. */\n  detectMode: (mode: EDialogProjectMode) => null,\n"
    );
  }
}
