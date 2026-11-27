use xrf_ltx::{Condlist, Section};

/// How the scripts read config text: names out of a list, a section's keys in the engine's order, and condlists.
pub(crate) struct ConfigText;

impl ConfigText {
  /// The names a list holds, as `utils.parse_names` reads them: runs of letters, digits, `_`, `-`, `.` and `\`,
  /// whatever separates them.
  pub(crate) fn split_names(value: &str) -> Vec<String> {
    value
      .split(|character: char| !(character.is_alphanumeric() || matches!(character, '_' | '-' | '.' | '\\')))
      .filter(|name| !name.is_empty())
      .map(String::from)
      .collect()
  }

  /// A section's keys in the order `CInifile::r_line` walks them: sorted by name, as the engine keeps them, a key
  /// written twice once (its last value).
  pub(crate) fn sorted_keys(section: &Section) -> Vec<&str> {
    let mut keys: Vec<&str> = section.iter().map(|(key, _)| key).collect();

    keys.sort_unstable();
    keys.dedup();
    keys
  }

  /// The condlist `value` holds, or why it does not parse.
  pub(crate) fn parse_condlist(value: &str) -> Result<Condlist, String> {
    Condlist::parse(value).map_err(|error| format!("'{value}' is no condlist: {error}"))
  }
}
