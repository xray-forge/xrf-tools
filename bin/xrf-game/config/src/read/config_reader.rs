use std::collections::BTreeMap;

use xrf_ltx::Section;

use crate::issue::config_issue::ConfigIssue;
use crate::issue::config_issues::ConfigIssues;
use crate::read::config_value::ConfigValue;

/// One section of a config, read the one way the game reads configs, every value typed and every key that does not read
/// collected as an issue naming its file, section and key.
///
/// The default policy, X-Ray's own: a key X-Ray reads only where it is there (`READ_IF_EXISTS`, `line_exist`) is read
/// with [`Self::read`] or [`Self::read_or`], its default taken silently where it is missing; a key X-Ray reads
/// unconditioned (`r_float` and the rest, which abort where it is missing) is read with [`Self::require`] or
/// [`Self::require_or`], an issue where it is missing. A key there but not of its kind is an issue either way. Where X-Ray
/// aborts, the game logs the issue and plays on with the default the reader names.
///
/// The config's text lives for `'f` and the issues are borrowed for `'i`, so text read out of it (a name, a key) is held
/// while the issues are handed on to the next reader.
pub struct ConfigReader<'f, 'i> {
  file: &'f str,
  name: &'f str,
  section: &'f Section,
  issues: &'i mut ConfigIssues,
}

impl<'f, 'i> ConfigReader<'f, 'i> {
  /// Reads `section`, named `name` in `file`, collecting into `issues`.
  pub(crate) fn new(file: &'f str, name: &'f str, section: &'f Section, issues: &'i mut ConfigIssues) -> Self {
    Self {
      file,
      name,
      section,
      issues,
    }
  }

  /// The section's name.
  pub fn get_name(&self) -> &'f str {
    self.name
  }

  /// Whether the section holds `key`.
  pub fn has(&self, key: &str) -> bool {
    self.section.get(key).is_some()
  }

  /// The text `key` holds, trimmed, none where the section lacks it: for a key whose value is a name list or may be
  /// empty, as a line with no `=` writes one.
  pub fn get_text(&self, key: &str) -> Option<&'f str> {
    self.section.get(key).map(str::trim)
  }

  /// The value of `key`, none where the section lacks it or, said as an issue, it is not of its kind.
  pub fn read<T: ConfigValue>(&mut self, key: &str) -> Option<T> {
    let text: &str = self.section.get(key)?;

    match T::parse(text) {
      Ok(value) => Some(value),
      Err(reason) => {
        self.push_key(key, reason);

        None
      }
    }
  }

  /// The value of `key`, `default` where the section lacks it or it is not of its kind.
  pub fn read_or<T: ConfigValue>(&mut self, key: &str, default: T) -> T {
    self.read(key).unwrap_or(default)
  }

  /// The value of `key`, read from `default` where the section lacks it, as X-Ray's scripts default a condlist's text
  /// (`spawn_point`, `self`); none, said as an issue, where the key's text is not of its kind.
  pub fn read_or_parse<T: ConfigValue>(&mut self, key: &str, default: &str) -> Option<T> {
    let text: &str = self.get_text(key).unwrap_or(default);

    match T::parse(text) {
      Ok(value) => Some(value),
      Err(reason) => {
        self.push_key(key, reason);

        None
      }
    }
  }

  /// The value of a key X-Ray reads unconditioned: none, said as an issue, where the section lacks it.
  pub fn require<T: ConfigValue>(&mut self, key: &str) -> Option<T> {
    if !self.has(key) {
      self.push_key(key, String::from("is missing"));

      return None;
    }

    self.read(key)
  }

  /// The value of a key X-Ray reads unconditioned, `default` where it is missing or not of its kind.
  pub fn require_or<T: ConfigValue>(&mut self, key: &str, default: T) -> T {
    self.require(key).unwrap_or(default)
  }

  /// The names the list `key` holds, as `utils.parse_names` reads them: runs of letters, digits, `_`, `-`, `.` and `\`,
  /// whatever separates them; none where the section lacks it.
  pub fn read_names(&self, key: &str) -> Vec<String> {
    self.section.get(key).map(Self::split_names).unwrap_or_default()
  }

  /// The section's keys in the order `CInifile::r_line` walks them: sorted by name, as the engine keeps them, a key
  /// written twice once.
  pub fn get_keys(&self) -> Vec<&'f str> {
    let mut keys: Vec<&str> = self.section.iter().map(|(key, _)| key).collect();

    keys.sort_unstable();
    keys.dedup();
    keys
  }

  /// Every key but those `except` names, each read as a `T`; one not of its kind is an issue and left out.
  pub fn read_all<T: ConfigValue>(&mut self, except: &[&str]) -> BTreeMap<String, T> {
    let mut values: BTreeMap<String, T> = BTreeMap::new();

    for key in self.get_keys() {
      if except.contains(&key) {
        continue;
      }

      if let Some(value) = self.read(key) {
        values.insert(String::from(key), value);
      }
    }

    values
  }

  /// Says an issue of the whole section.
  pub fn push_issue(&mut self, message: impl Into<String>) {
    self.issues.push(ConfigIssue {
      file: String::from(self.file),
      section: Some(String::from(self.name)),
      key: None,
      message: message.into(),
    });
  }

  /// Says an issue of `key`.
  pub fn push_key(&mut self, key: &str, message: impl Into<String>) {
    self.issues.push(ConfigIssue {
      file: String::from(self.file),
      section: Some(String::from(self.name)),
      key: Some(String::from(key)),
      message: message.into(),
    });
  }

  /// The names `value` holds, as [`Self::read_names`] splits them.
  pub fn split_names(value: &str) -> Vec<String> {
    value
      .split(|character: char| !(character.is_alphanumeric() || matches!(character, '_' | '-' | '.' | '\\')))
      .filter(|name| !name.is_empty())
      .map(String::from)
      .collect()
  }
}
