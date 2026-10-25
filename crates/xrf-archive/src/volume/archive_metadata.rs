/// The `[header]` text a volume's metadata chunk carries, which names where its entries unpack.
pub(crate) struct ArchiveMetadata;

impl ArchiveMetadata {
  /// The `[header] entry_point` of decoded metadata `text` without its leading `$alias$\`, the root alias an unpack
  /// does not recreate, or `None` when the header names no entry point.
  // todo: read the metadata as `CInifile::Load` does: strip `;` and `//` comments outside quotes, open a section only on
  // a line starting with `[` and name it up to its first `]` in any case, and trim and unquote the value.
  pub(crate) fn find_root_path(text: &str) -> Option<&str> {
    let mut section: &str = "";

    for line in text.lines() {
      match (Self::parse_section_name(line), section) {
        (None, "header") => {
          if let Some(("entry_point", value)) = Self::parse_variable(line) {
            return Some(Self::strip_root_alias(value));
          }
        }
        (Some(name), _) => {
          section = name;
        }
        _ => {}
      }
    }

    None
  }

  /// The name of a line ending in `[name]`, whatever precedes the bracket.
  fn parse_section_name(line: &str) -> Option<&str> {
    let (_, name) = line.strip_suffix(']')?.rsplit_once('[')?;

    name.chars().all(Self::is_word_char).then_some(name)
  }

  /// The `name = value` of a line, the value with its leading whitespace trimmed and its trailing whitespace kept.
  fn parse_variable(line: &str) -> Option<(&str, &str)> {
    let (name, value) = line.split_once('=')?;
    let name: &str = name.trim();
    let value: &str = value.trim_start();

    (!name.is_empty() && name.chars().all(Self::is_word_char) && !value.is_empty()).then_some((name, value))
  }

  /// The entry point without its leading `$alias$\`.
  fn strip_root_alias(entry_point: &str) -> &str {
    entry_point
      .strip_prefix('$')
      .and_then(|rest| rest.split_once('$'))
      .filter(|(alias, _)| !alias.is_empty() && alias.chars().all(Self::is_word_char))
      .and_then(|(_, path)| path.strip_prefix('\\'))
      .unwrap_or(entry_point)
  }

  fn is_word_char(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
  }
}

#[cfg(test)]
mod tests {
  use super::ArchiveMetadata;

  #[test]
  fn the_entry_point_loses_its_root_alias() {
    assert_eq!(
      ArchiveMetadata::find_root_path("[header]\nauto_load = true\nentry_point = $game_data$\\levels\\\n"),
      Some("levels\\")
    );
    assert_eq!(
      ArchiveMetadata::find_root_path("[header]\n  entry_point=gamedata"),
      Some("gamedata")
    );
    assert_eq!(
      ArchiveMetadata::find_root_path("[header]\r\nentry_point = $fs_root$\\\r\n"),
      Some("")
    );
  }

  #[test]
  fn an_entry_point_outside_the_header_section_is_ignored() {
    assert_eq!(
      ArchiveMetadata::find_root_path("entry_point = gamedata\n[header]\nauto_load = true"),
      None
    );
    assert_eq!(
      ArchiveMetadata::find_root_path("[header]\n[other]\nentry_point = gamedata"),
      None
    );
  }

  /// Unlike `CInifile`, which drops the comment and so never opens the section.
  #[test]
  fn a_comment_ending_in_a_section_name_opens_that_section() {
    assert_eq!(
      ArchiveMetadata::find_root_path("; comment [header]\nentry_point = gamedata"),
      Some("gamedata")
    );
  }

  #[test]
  fn header_lines_split_into_sections_variables_and_aliases() {
    assert_eq!(ArchiveMetadata::parse_section_name("[header]"), Some("header"));
    assert_eq!(ArchiveMetadata::parse_section_name("[]"), Some(""));
    assert_eq!(ArchiveMetadata::parse_section_name("[head er]"), None);
    assert_eq!(ArchiveMetadata::parse_section_name("[header] "), None);

    assert_eq!(
      ArchiveMetadata::parse_variable(" name = a = b "),
      Some(("name", "a = b "))
    );
    assert_eq!(ArchiveMetadata::parse_variable("name ="), None);
    assert_eq!(ArchiveMetadata::parse_variable("two words = value"), None);

    assert_eq!(ArchiveMetadata::strip_root_alias("$game_data$\\meshes"), "meshes");
    assert_eq!(ArchiveMetadata::strip_root_alias("$$\\meshes"), "$$\\meshes");
    assert_eq!(
      ArchiveMetadata::strip_root_alias("$game data$\\meshes"),
      "$game data$\\meshes"
    );
    assert_eq!(
      ArchiveMetadata::strip_root_alias("$game_data$meshes"),
      "$game_data$meshes"
    );
  }
}
