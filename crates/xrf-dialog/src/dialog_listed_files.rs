use xrf_extension::XrayExtension;

/// The dialog and partner files the engine loads, named as `system.ltx` lists them: the `files` of `[dialogs]`,
/// `[profiles]` and `[info_portions]`, and `specific_characters_files` of `[profiles]`, each a name under the dialogs
/// prefix without its `.xml`.
///
/// `CXML_IdToIndex::InitInternal` reads these and only these, in list order, so a file the lists leave out declares
/// nothing to the engine whatever its name.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DialogListedFiles {
  pub dialogs: Vec<String>,
  pub characters: Vec<String>,
  pub profiles: Vec<String>,
  pub info_portions: Vec<String>,
}

impl DialogListedFiles {
  /// Splits one list value the way the engine reads it: comma separated, each name trimmed, empty names skipped.
  pub fn split_list(value: &str) -> Vec<String> {
    value
      .split(',')
      .map(str::trim)
      .filter(|name| !name.is_empty())
      .map(str::to_owned)
      .collect()
  }

  /// The logical path the engine opens a listed name at: `<prefix>\<name>.xml`.
  pub fn resolve_logical_path(prefix: &str, name: &str) -> String {
    format!("{prefix}\\{name}.{}", XrayExtension::Xml)
  }

  /// Whether a logical path is one of the listed dialog files, matched as the case-insensitive file system the engine
  /// opens them on does.
  pub fn is_dialog_file(&self, prefix: &str, logical_path: &str) -> bool {
    Self::contains(&self.dialogs, prefix, logical_path)
  }

  /// Whether a logical path is one of the listed info portion files.
  pub fn is_info_portion_file(&self, prefix: &str, logical_path: &str) -> bool {
    Self::contains(&self.info_portions, prefix, logical_path)
  }

  /// Whether a logical path is one of the listed character, profile or info portion files.
  pub fn is_partner_file(&self, prefix: &str, logical_path: &str) -> bool {
    [&self.characters, &self.profiles, &self.info_portions]
      .into_iter()
      .any(|names| Self::contains(names, prefix, logical_path))
  }

  fn contains(names: &[String], prefix: &str, logical_path: &str) -> bool {
    names
      .iter()
      .any(|name| Self::resolve_logical_path(prefix, name).eq_ignore_ascii_case(logical_path))
  }
}
