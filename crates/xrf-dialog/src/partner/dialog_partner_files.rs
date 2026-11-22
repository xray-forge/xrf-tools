/// The partner files the engine loads, named as `system.ltx` lists them: `specific_characters_files` and the `files` of
/// `[profiles]` and `[info_portions]`, each a name under the dialogs prefix without its `.xml`.
///
/// `CXML_IdToIndex::InitInternal` reads these and only these, in list order, so a file the lists leave out declares
/// nothing to the engine whatever its name.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DialogPartnerFiles {
  pub characters: Vec<String>,
  pub profiles: Vec<String>,
  pub info_portions: Vec<String>,
}

impl DialogPartnerFiles {
  /// Splits one list value the way the engine reads it: comma separated, each name trimmed, empty names skipped.
  pub fn split_list(value: &str) -> Vec<String> {
    value
      .split(',')
      .map(str::trim)
      .filter(|name| !name.is_empty())
      .map(str::to_owned)
      .collect()
  }
}
