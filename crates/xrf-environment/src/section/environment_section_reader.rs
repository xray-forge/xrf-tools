use xrf_engine_target::XrayEngine;
use xrf_ltx::{LtxProvenance, Section, read_engine_bool, read_engine_floats, read_engine_integer, scan_engine_float};

use crate::finding::{EnvironmentFinding, EnvironmentRule};
use crate::key::{EnvironmentKey, EnvironmentKeyUse, EnvironmentValue, EnvironmentValueKind};
use crate::section::environment_origin::EnvironmentOrigin;
use crate::section::environment_section::EnvironmentSection;

/// Reads sections of one config against a key table, as one engine reads them, and says what that engine would refuse
/// or misread.
pub(crate) struct EnvironmentSectionReader<'a> {
  pub engine: XrayEngine,
  /// How messages name the kind of section: `Weather`, `Sun`, `Thunderbolt`.
  pub subject: &'static str,
  /// The config the sections are read from, as a logical path.
  pub file: &'a str,
  /// Where each key came from, where the read records it.
  pub provenance: Option<&'a LtxProvenance>,
  pub findings: &'a mut Vec<EnvironmentFinding>,
}

impl<'a> EnvironmentSectionReader<'a> {
  /// The words `r_bool` is written with, either way; any other is read as false, which is worth saying.
  const FLAG_WORDS: [&'static str; 8] = ["on", "yes", "true", "1", "off", "no", "false", "0"];

  pub fn get_engine(&self) -> XrayEngine {
    self.engine
  }

  pub fn get_file(&self) -> &'a str {
    self.file
  }

  /// Reads one section: each key of the table as the engine parses it, the rest as written, and a finding for each
  /// key the engine requires without a condition that the section does not write.
  pub fn read<K: EnvironmentKey>(&mut self, name: &str, section: &Section) -> EnvironmentSection<K> {
    let mut read: EnvironmentSection<K> = EnvironmentSection::new(name, self.file);

    for (key_name, raw) in section.iter() {
      if let Some(origin) = self.provenance.and_then(|provenance| provenance.get(name, key_name)) {
        read
          .origins
          .insert(key_name.to_owned(), EnvironmentOrigin::from(origin));
      }

      match K::from_name(key_name) {
        Some(key) => {
          let value: EnvironmentValue = self.parse(name, key, raw);

          read.values.insert(key, value);
        }
        None => {
          read.extras.insert(key_name.to_owned(), raw.to_owned());
        }
      }
    }

    for key in K::ALL.iter().copied() {
      if matches!(key.get_use(self.engine), EnvironmentKeyUse::Required(_)) && !read.has(key) {
        self.report_missing(name, key.get_name());
      }
    }

    read
  }

  /// Says the engine requires a key the section does not write, for a key whose requirement another key decides.
  pub fn require<K: EnvironmentKey>(&mut self, section: &EnvironmentSection<K>, key: K) {
    if !section.has(key) {
      self.report_missing(&section.name, key.get_name());
    }
  }

  /// Reports one finding about a section of this config.
  pub fn report(&mut self, rule: EnvironmentRule, section: &str, key: Option<&str>, message: String) {
    self.findings.push(EnvironmentFinding {
      file: self.file.to_owned(),
      key: key.map(str::to_owned),
      message,
      rule,
      section: Some(section.to_owned()),
    });
  }

  /// Reports one finding about this config as a whole.
  pub fn report_file(&mut self, rule: EnvironmentRule, message: String) {
    self.findings.push(EnvironmentFinding {
      file: self.file.to_owned(),
      key: None,
      message,
      rule,
      section: None,
    });
  }

  /// The subject and section as messages name them: `Weather [12:00:00]`.
  pub fn describe(&self, section: &str) -> String {
    format!("{} [{section}]", self.subject)
  }

  fn report_missing(&mut self, section: &str, key: &str) {
    let message: String = format!("{} is missing required field [{key}]", self.describe(section));

    self.report(EnvironmentRule::Engine, section, Some(key), message);
  }

  fn parse<K: EnvironmentKey>(&mut self, section: &str, key: K, raw: &str) -> EnvironmentValue {
    let value: &str = raw.trim();
    // A key the engine does not read cannot be misread by it, so it is kept and not judged.
    let is_read: bool = key.get_use(self.engine).is_read();
    let name: &str = key.get_name();

    match key.get_kind() {
      EnvironmentValueKind::Number => {
        let number: Option<(f32, usize)> = scan_engine_float(value);

        match number {
          _ if !is_read => {}
          None => {
            let message: String = format!(
              "{} has [{name}] = '{value}', which is not a number; the engine reads it as 0",
              self.describe(section)
            );

            self.report(EnvironmentRule::Engine, section, Some(name), message);
          }
          // `atof` reads the number and stops, so `0.25f` is the 0.25 it was meant to be: written loosely, not misread.
          Some((read, used)) if used != value.len() => {
            let message: String = format!(
              "{} has [{name}] = '{value}', whose trailing text the engine ignores, reading {read}",
              self.describe(section)
            );

            self.report(EnvironmentRule::Convention, section, Some(name), message);
          }
          Some(_) => {}
        }

        EnvironmentValue::Number(number.map_or(0.0, |(number, _)| number))
      }
      EnvironmentValueKind::Integer => {
        let integer: i32 = read_engine_integer(value);

        if is_read && value.parse::<i32>().is_err() {
          let message: String = format!(
            "{} has [{name}] = '{value}', which is not a whole number; the engine reads it as {integer}",
            self.describe(section)
          );

          self.report(EnvironmentRule::Engine, section, Some(name), message);
        }

        EnvironmentValue::Integer(integer)
      }
      EnvironmentValueKind::Vector { .. } => EnvironmentValue::Vector(self.read_vector(section, key, value)),
      EnvironmentValueKind::Flag => {
        if is_read && !Self::FLAG_WORDS.iter().any(|word| value.eq_ignore_ascii_case(word)) {
          let message: String = format!(
            "{} has [{name}] = '{value}', which the engine reads as false",
            self.describe(section)
          );

          self.report(EnvironmentRule::Convention, section, Some(name), message);
        }

        EnvironmentValue::Flag(read_engine_bool(value))
      }
      EnvironmentValueKind::Text => EnvironmentValue::Text(value.to_owned()),
      EnvironmentValueKind::List => EnvironmentValue::List(Self::split_list(value)),
    }
  }

  /// A vector as `sscanf` reads it, saying what it made of one not written as the engine reads one.
  fn read_vector<K: EnvironmentKey>(&mut self, section: &str, key: K, value: &str) -> Vec<f32> {
    let EnvironmentValueKind::Vector { least, most } = key.get_kind() else {
      return Vec::new();
    };
    let components: Vec<f32> = read_engine_floats(value, usize::from(most));

    // A key the engine does not read cannot be misread by it.
    if !key.get_use(self.engine).is_read() {
      return components;
    }

    let name: &str = key.get_name();
    let written: usize = if value.is_empty() { 0 } else { value.split(',').count() };
    let is_clean: bool = value
      .split(',')
      .take(usize::from(most))
      .all(|token| token.trim().parse::<f32>().is_ok());

    if components.len() < usize::from(least) || !is_clean {
      let message: String = format!(
        "{} has [{name}] = '{value}'; the engine reads {} of {least} components from it and leaves the rest at zero",
        self.describe(section),
        components.len().min(usize::from(least)),
      );

      self.report(EnvironmentRule::Engine, section, Some(name), message);
    } else if written > usize::from(most) {
      let message: String = format!(
        "{} has [{name}] = '{value}' with {written} components; the engine reads the first {most}",
        self.describe(section)
      );

      self.report(EnvironmentRule::Convention, section, Some(name), message);
    }

    components
  }

  /// A list as `_GetItemCount` and `_GetItem` walk it: comma-separated, each item trimmed, none in empty text.
  fn split_list(value: &str) -> Vec<String> {
    if value.is_empty() {
      return Vec::new();
    }

    value.split(',').map(|item| item.trim().to_owned()).collect()
  }
}
