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

        let mut is_escaped: bool = false;

        for skipped in characters.by_ref() {
          blank(&mut stripped, skipped);

          // An escaped quote is part of the literal, which `render_string` and `render_value` write for a spelling
          // or a constant holding one.
          if !is_escaped && skipped == character {
            break;
          }

          is_escaped = !is_escaped && skipped == '\\';
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

#[cfg(test)]
mod tests {
  use super::{for_each_identifier, without_comments_and_strings};

  fn identifiers(source: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();

    for_each_identifier(&without_comments_and_strings(source), |_, _, identifier| {
      found.push(identifier.to_owned())
    });

    found
  }

  #[test]
  fn blanks_comments_and_strings_byte_for_byte() {
    let source: &str = "/** Shape */ type A = \"B\" | 'C' | `D`; // E\nF";
    let blanked: String = without_comments_and_strings(source);

    assert_eq!(blanked.len(), source.len());
    assert_eq!(identifiers(source), ["type", "A", "F"]);
  }

  #[test]
  fn an_escaped_quote_does_not_end_its_string() {
    // A constant holding `say "Shape" now`, as `render_value` writes it: nothing inside names a type.
    assert_eq!(
      identifiers("export const NOTE = \"say \\\"Shape\\\" now\" as const;"),
      ["export", "const", "NOTE", "as", "const"]
    );
    // An escaped backslash before the closing quote does end it.
    assert_eq!(identifiers("type A = \"path\\\\\" | Shape;"), ["type", "A", "Shape"]);
  }

  #[test]
  fn a_property_name_is_no_reference() {
    assert_eq!(identifiers("{ Sphere: Shape; Cube?: Shape }"), ["Shape", "Shape"]);
  }
}
