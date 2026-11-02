/// The `[options] exclude_exts` dialect a packing and a patching configuration share.
///
/// A pattern is matched against the extension of a name's last component with its dot, without case, through the `*`
/// and `?` wildcards of xrCompress's `PatternMatch`. Both configurations read the field from the same LTX key and the
/// same JSON name, so one pattern means one thing whichever run it is handed to.
pub(crate) struct ArchiveExtensionPattern;

impl ArchiveExtensionPattern {
  /// Whether any of `patterns` matches the extension of `name`.
  pub(crate) fn is_any_matched(name: &str, patterns: &[String]) -> bool {
    if patterns.is_empty() {
      return false;
    }

    let extension: String = Self::extension_of(&name.to_ascii_lowercase()).to_owned();

    patterns.iter().any(|pattern| Self::matches(&extension, pattern))
  }

  /// The extension of a name's last component, with its dot, or empty when it has none.
  pub(crate) fn extension_of(name: &str) -> &str {
    let file: &str = name.rsplit(['\\', '/']).next().unwrap_or(name);

    file.rfind('.').map_or("", |dot| &file[dot..])
  }

  /// Match a `*` and `?` wildcard pattern, case-insensitively, the way `PatternMatch` does.
  pub(crate) fn matches(text: &str, pattern: &str) -> bool {
    let text: Vec<char> = text.to_ascii_lowercase().chars().collect();
    let pattern: Vec<char> = pattern.to_ascii_lowercase().chars().collect();

    let mut text_index: usize = 0;
    let mut pattern_index: usize = 0;
    let mut star: Option<(usize, usize)> = None;

    while text_index < text.len() {
      match pattern.get(pattern_index) {
        Some('*') => {
          star = Some((pattern_index, text_index));
          pattern_index += 1;
        }
        Some('?') => {
          text_index += 1;
          pattern_index += 1;
        }
        Some(symbol) if *symbol == text[text_index] => {
          text_index += 1;
          pattern_index += 1;
        }
        // Backtrack to the last star and let it swallow one more character.
        _ => match star {
          Some((star_index, matched)) => {
            pattern_index = star_index + 1;
            text_index = matched + 1;
            star = Some((star_index, text_index));
          }
          None => return false,
        },
      }
    }

    pattern[pattern_index..].iter().all(|symbol| *symbol == '*')
  }
}

#[cfg(test)]
mod tests {
  use super::ArchiveExtensionPattern;

  fn patterns(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| String::from(*value)).collect()
  }

  #[test]
  fn matches_wildcards_case_insensitively() {
    assert!(ArchiveExtensionPattern::matches(".txt", "*.txt"));
    assert!(ArchiveExtensionPattern::matches(".TXT", "*.txt"));
    assert!(ArchiveExtensionPattern::matches(".json", "*.js?n"));
    assert!(ArchiveExtensionPattern::matches(".anything", "*"));
    assert!(!ArchiveExtensionPattern::matches(".ltx", "*.txt"));
    assert!(!ArchiveExtensionPattern::matches("", "*.txt"));
  }

  #[test]
  fn an_extension_is_the_last_components_with_its_dot() {
    assert_eq!(ArchiveExtensionPattern::extension_of("configs\\system.ltx"), ".ltx");
    assert_eq!(ArchiveExtensionPattern::extension_of("archive.tar.gz"), ".gz");
    assert_eq!(
      ArchiveExtensionPattern::extension_of("configs.d\\readme"),
      "",
      "a dotted directory is not it"
    );
    assert_eq!(ArchiveExtensionPattern::extension_of("configs\\system."), ".");
  }

  #[test]
  fn a_name_is_matched_by_its_extension_alone() {
    // A pattern reaches only the extension, so one naming more of the name than that matches nothing.
    assert!(ArchiveExtensionPattern::is_any_matched(
      "Configs\\Data.JSON",
      &patterns(&["*.js?n"])
    ));
    assert!(!ArchiveExtensionPattern::is_any_matched(
      "archive.tar.gz",
      &patterns(&["*.tar.gz"])
    ));
    assert!(!ArchiveExtensionPattern::is_any_matched("configs\\system.ltx", &[]));
  }
}
