//! Reading generated TypeScript back as text, which is how a rendered module's dependencies are recovered.
//!
//! Specta renders each module in isolation, so the imports tying them together cannot come from the type graph.

/// Generated source with doc comments and string literals blanked out, byte for byte.
pub(crate) fn without_comments_and_strings(source: &str) -> String {
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
pub(crate) fn for_each_identifier(blanked: &str, mut visit: impl FnMut(usize, usize, &str)) {
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
