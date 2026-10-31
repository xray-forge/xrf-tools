use crate::volume::ArchiveHeaderItem;

/// The `[header]` text a volume's metadata chunk carries, which names where its entries unpack, read with the lexical
/// rules `CInifile::Load` applies to it.
pub struct ArchiveMetadata;

impl ArchiveMetadata {
  /// The section name every reader of the metadata asks for.
  const SECTION: &str = "header";

  /// The `[header]` items of decoded metadata `text` as the engine loads them, a later line replacing an earlier one of
  /// the same key, or `None` when no line opens `[header]`.
  ///
  /// Each line loses its comment, and a line opening with `[` names a section up to its first `]`, without case. A
  /// value with an odd quote, which the engine continues onto the next lines, is read from its own line alone.
  pub fn read_header(text: &str) -> Option<Vec<ArchiveHeaderItem>> {
    let mut items: Option<Vec<ArchiveHeaderItem>> = None;
    let mut is_in_header: bool = false;

    for line in text.lines() {
      let line: &str = Self::strip_comment(line.trim());

      if let Some(rest) = line.strip_prefix('[') {
        is_in_header = rest
          .split_once(']')
          .is_some_and(|(name, _)| name.eq_ignore_ascii_case(Self::SECTION));

        if is_in_header {
          items.get_or_insert_with(Vec::new);
        }

        continue;
      }

      let Some(items) = items.as_mut().filter(|_| is_in_header) else {
        continue;
      };
      let (key, value) = line.split_once('=').unwrap_or((line, ""));
      let item: ArchiveHeaderItem = ArchiveHeaderItem {
        key: key.trim().to_owned(),
        value: Self::strip_unquoted_whitespace(value),
      };

      // A line with no name is dropped, as a read-only `CInifile` drops it.
      if item.key.is_empty() {
        continue;
      }

      match items.iter_mut().find(|existing| existing.key == item.key) {
        Some(existing) => *existing = item,
        None => items.push(item),
      }
    }

    items
  }

  /// The `[header] entry_point` of decoded metadata `text` without its leading `$alias$\`, the root alias an unpack
  /// does not recreate, or `None` when the header names no entry point.
  pub(crate) fn find_root_path(text: &str) -> Option<String> {
    Self::read_header(text)?
      .into_iter()
      .find(|item| item.key == "entry_point" && !item.value.is_empty())
      .map(|item| Self::strip_root_alias(&item.value).to_owned())
  }

  /// The directory under `$fs_root$` the engine registers a volume's entries in, given the entry point
  /// [`Self::find_root_path`] found: the alias always stands for `$fs_root$` there, and a volume without metadata
  /// mounts at `gamedata` (`CLocatorAPI::LoadArchive`). Separators are backslashes, with none at either end.
  pub(crate) fn to_mount_root(root_path: Option<&str>) -> String {
    root_path.map_or_else(
      || String::from("gamedata"),
      |root| root.replace('/', "\\").trim_matches('\\').to_owned(),
    )
  }

  /// A line without its comment: from the first `;`, or from the first `/` when a second follows it. A comment
  /// character between a pair of quotes is text, and then the line is kept whole.
  fn strip_comment(line: &str) -> &str {
    let semicolon: Option<usize> = line.find(';');
    let slashes: Option<usize> = line.find('/').filter(|&slash| line[slash + 1..].starts_with('/'));
    let Some(start) = semicolon.into_iter().chain(slashes).min() else {
      return line;
    };
    let is_quoted: bool = line[..start]
      .find('"')
      .is_some_and(|open| line[open + 1..].find('"').is_some_and(|close| open + 1 + close > start));

    if is_quoted { line } else { &line[..start] }
  }

  /// A value as `_parse` keeps it: whitespace outside quotes removed wherever it stands, the quotes themselves kept.
  fn strip_unquoted_whitespace(value: &str) -> String {
    let mut is_quoted: bool = false;

    value
      .chars()
      .filter(|&character| {
        if character == '"' {
          is_quoted = !is_quoted;
        }

        is_quoted || !matches!(character, ' ' | '\t' | '\n' | '\x0B' | '\x0C' | '\r')
      })
      .collect()
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
  use crate::volume::ArchiveHeaderItem;

  fn root_of(text: &str) -> Option<String> {
    ArchiveMetadata::find_root_path(text)
  }

  #[test]
  fn the_entry_point_loses_its_root_alias() {
    assert_eq!(
      root_of("[header]\nauto_load = true\nentry_point = $game_data$\\levels\\\n").as_deref(),
      Some("levels\\")
    );
    assert_eq!(root_of("[header]\n  entry_point=gamedata").as_deref(), Some("gamedata"));
    assert_eq!(
      root_of("[header]\r\nentry_point = $fs_root$\\\r\n").as_deref(),
      Some("")
    );
  }

  #[test]
  fn a_commented_header_reads_as_the_uncommented_one() {
    let plain: Option<String> = root_of("[header]\nauto_load = true\nentry_point = $fs_root$\\gamedata\\\n");

    assert_eq!(plain.as_deref(), Some("gamedata\\"));

    for text in [
      "[header] ; archive metadata\nauto_load = true\nentry_point = $fs_root$\\gamedata\\\n",
      "[header]\nauto_load = true\nentry_point = $fs_root$\\gamedata\\ ; installed content\n",
      "[header] // archive metadata\nentry_point = $fs_root$\\gamedata\\// installed content\n",
      "; leading comment\n[Header]\n  entry_point =   $fs_root$\\gamedata\\   \n",
    ] {
      assert_eq!(root_of(text), plain, "{text:?}");
    }
  }

  #[test]
  fn a_comment_ending_in_a_section_name_opens_nothing() {
    assert_eq!(root_of("; comment [header]\nentry_point = gamedata"), None);
    assert_eq!(ArchiveMetadata::read_header("; [header]"), None);
  }

  #[test]
  fn an_entry_point_outside_the_header_section_is_ignored() {
    assert_eq!(root_of("entry_point = gamedata\n[header]\nauto_load = true"), None);
    assert_eq!(root_of("[header]\n[other]\nentry_point = gamedata"), None);
  }

  #[test]
  fn a_later_entry_point_replaces_an_earlier_one_and_an_empty_one_names_none() {
    assert_eq!(
      root_of("[header]\nentry_point = gamedata\nentry_point = $fs_root$\\levels\\").as_deref(),
      Some("levels\\")
    );
    assert_eq!(root_of("[header]\nentry_point = ; nothing"), None);
    // Keys keep their case, as `CInifile` compares them.
    assert_eq!(root_of("[header]\nEntry_Point = gamedata"), None);
  }

  #[test]
  fn values_lose_unquoted_whitespace_and_keep_quoted_comments() {
    assert_eq!(
      ArchiveMetadata::read_header("[header]\ncreator = \"GSC; Game World\" ; studio\nlevel name = a b\nflag\n"),
      Some(vec![
        ArchiveHeaderItem {
          key: String::from("creator"),
          // The first `;` sits inside the quotes, so the engine keeps the line whole, its trailing comment included.
          value: String::from("\"GSC; Game World\";studio"),
        },
        ArchiveHeaderItem {
          key: String::from("level name"),
          value: String::from("ab"),
        },
        ArchiveHeaderItem {
          key: String::from("flag"),
          value: String::new(),
        },
      ])
    );
  }

  #[test]
  fn a_single_slash_does_not_open_a_comment() {
    assert_eq!(
      ArchiveMetadata::strip_comment("entry_point = $fs_root$/gamedata/ ; x"),
      "entry_point = $fs_root$/gamedata/ "
    );
    // The engine only looks at the first slash, so a later pair behind a lone one is text.
    assert_eq!(ArchiveMetadata::strip_comment("a = b/c//d"), "a = b/c//d");
  }

  #[test]
  fn a_volume_mounts_under_the_file_system_root_or_at_gamedata_without_metadata() {
    assert_eq!(
      ArchiveMetadata::to_mount_root(Some("gamedata\\configs\\")),
      "gamedata\\configs"
    );
    assert_eq!(ArchiveMetadata::to_mount_root(Some("levels/")), "levels");
    assert_eq!(ArchiveMetadata::to_mount_root(Some("gamedata")), "gamedata");
    assert_eq!(ArchiveMetadata::to_mount_root(Some("")), "");
    assert_eq!(ArchiveMetadata::to_mount_root(None), "gamedata");
  }

  #[test]
  fn an_alias_is_stripped_only_when_it_is_one() {
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
