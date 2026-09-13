//! Reading generated TypeScript back, to learn which declarations a block of it depends on.
//!
//! Specta renders each module in isolation, so the imports tying the modules together have to be recovered
//! from the rendered text rather than from the type graph.

use std::collections::{BTreeMap, BTreeSet};

use crate::ipc::bindings::constants::{BINDINGS_ROOT, TYPES_DIRECTORY};

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

/// Calls `visit` with the byte range of every identifier in `source` that is not a property name.
///
/// Both readers of the scan share it, so the set of names an import is written for and the set a rewrite
/// replaces can never disagree about what counts as a reference.
fn for_each_identifier(source: &str, mut visit: impl FnMut(usize, usize, &str)) {
  let blanked: String = without_comments_and_strings(source);
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
pub(super) fn referenced_types<'a>(
  source: &str,
  owners: &'a BTreeMap<String, String>,
  owner: &str,
) -> BTreeSet<&'a str> {
  let mut referenced: BTreeSet<&str> = BTreeSet::new();

  for_each_identifier(source, |_, _, identifier| {
    if let Some((name, module)) = owners.get_key_value(identifier)
      && module != owner
    {
      referenced.insert(name.as_str());
    }
  });

  referenced
}

/// `source` with every type reference `renames` covers replaced by the name it maps to.
///
/// Replacing text rather than rewriting a parsed module: the generated modules carry the Rust
/// doc comments, which SWC keeps outside the tree and a re-render would drop. This reuses the same identifier
/// scan the imports are computed from, so a reference is rewritten exactly when it would have been imported,
/// and prose naming a type in a doc comment is left alone.
pub(super) fn rewrite_type_references(source: &str, renames: &BTreeMap<String, String>) -> String {
  let mut rewritten: String = String::with_capacity(source.len());
  let mut copied: usize = 0;

  for_each_identifier(source, |start, end, identifier| {
    if let Some(replacement) = renames.get(identifier) {
      rewritten.push_str(&source[copied..start]);
      rewritten.push_str(replacement);
      copied = end;
    }
  });

  rewritten.push_str(&source[copied..]);
  rewritten
}

/// Import statements pulling every referenced type from the module that declares it.
pub(super) fn render_imports(referenced: &BTreeSet<&str>, owners: &BTreeMap<String, String>) -> String {
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
