use std::fmt::{Display, Formatter};

use serde::Serialize;

use crate::dialect::LtxResolutionDiagnostic;
use crate::finding::LtxFindingKind;

/// One thing wrong with a config tree, placed at the entry point it was found under and the config it is about.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxFinding {
  pub kind: LtxFindingKind,
  /// The entry point whose resolution the finding was produced under.
  pub entry: String,
  /// The config to open, where one is known.
  pub file: Option<String>,
  pub section: Option<String>,
  /// The field it is about, or `None` for a finding about the section as a whole.
  pub field: Option<String>,
  pub message: String,
  /// What the engine does with the same input, where the dialect said so.
  pub engine_behaviour: Option<String>,
}

impl LtxFinding {
  /// A section, or one of its fields, that breaks the scheme it is bound to.
  pub fn new_scheme<E, S, M>(entry: E, file: Option<String>, section: S, field: Option<&str>, message: M) -> Self
  where
    E: Into<String>,
    S: Into<String>,
    M: Into<String>,
  {
    Self {
      engine_behaviour: None,
      entry: entry.into(),
      field: field.map(String::from),
      file,
      kind: LtxFindingKind::Scheme,
      message: message.into(),
      section: Some(section.into()),
    }
  }

  /// An entry point whose resolution failed, carrying why.
  pub fn new_unreadable<E, M>(entry: E, message: M) -> Self
  where
    E: Into<String>,
    M: Into<String>,
  {
    Self {
      engine_behaviour: None,
      entry: entry.into(),
      field: None,
      file: None,
      kind: LtxFindingKind::Unreadable,
      message: message.into(),
      section: None,
    }
  }

  /// What the dialect said while resolving `entry`, placed at the file it blamed or, failing that, at `declared_in`.
  ///
  /// A diagnostic about keys above any header names no section, and says so with an empty one.
  pub fn from_diagnostic(entry: &str, diagnostic: &LtxResolutionDiagnostic, declared_in: Option<&str>) -> Self {
    Self {
      engine_behaviour: diagnostic.engine_behaviour.clone(),
      entry: String::from(entry),
      field: None,
      file: diagnostic.file.clone().or_else(|| declared_in.map(String::from)),
      kind: LtxFindingKind::Dialect,
      message: diagnostic.message.clone(),
      section: Some(diagnostic.section.clone()).filter(|section| !section.is_empty()),
    }
  }

  /// The message preceded by the section and field it is about, for a surface with one column to say it in.
  pub fn get_located_message(&self) -> String {
    match (&self.section, &self.field) {
      (Some(section), Some(field)) => format!("[{section}] {field}: {}", self.message),
      (Some(section), None) => format!("[{section}]: {}", self.message),
      (None, _) => self.message.clone(),
    }
  }
}

impl Display for LtxFinding {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    formatter.write_str(self.kind.get_label())?;

    match &self.file {
      Some(file) if *file != self.entry => write!(formatter, " in '{file}' resolved from '{}'", self.entry)?,
      _ => write!(formatter, " in '{}'", self.entry)?,
    }

    if let Some(section) = &self.section {
      write!(formatter, " [{section}]")?;
    }

    if let Some(field) = &self.field {
      write!(formatter, " {field}")?;
    }

    write!(formatter, " : {}", self.message)
  }
}

#[cfg(test)]
mod tests {
  use super::LtxFinding;
  use crate::LtxResolutionDiagnostic;

  #[test]
  fn names_the_declaring_config_beside_the_entry_point_that_reached_it() {
    let finding: LtxFinding = LtxFinding::new_scheme(
      "configs/system.ltx",
      Some(String::from("configs/items/w_broken.ltx")),
      "section",
      Some("field"),
      "message",
    );

    assert_eq!(
      finding.to_string(),
      "Ltx scheme error in 'configs/items/w_broken.ltx' resolved from 'configs/system.ltx' [section] field : message"
    );
    assert_eq!(finding.get_located_message(), "[section] field: message");
  }

  #[test]
  fn names_one_file_once_when_the_section_is_written_in_its_entry_point() {
    let finding: LtxFinding = LtxFinding::new_scheme(
      "configs/system.ltx",
      Some(String::from("configs/system.ltx")),
      "section",
      None,
      "message",
    );

    assert_eq!(
      finding.to_string(),
      "Ltx scheme error in 'configs/system.ltx' [section] : message"
    );
    assert_eq!(finding.get_located_message(), "[section]: message");
  }

  #[test]
  fn names_the_entry_point_alone_when_nothing_said_where_the_section_was_declared() {
    let finding: LtxFinding = LtxFinding::new_scheme("configs/system.ltx", None, "section", Some("field"), "message");

    assert_eq!(
      finding.to_string(),
      "Ltx scheme error in 'configs/system.ltx' [section] field : message"
    );
  }

  #[test]
  fn reports_an_unreadable_entry_point_by_the_entry_point() {
    let finding: LtxFinding = LtxFinding::new_unreadable("configs/system.ltx", "Ltx parse error: 4:1 broken");

    assert_eq!(
      finding.to_string(),
      "Ltx unreadable entry in 'configs/system.ltx' : Ltx parse error: 4:1 broken"
    );
    assert_eq!(finding.get_located_message(), "Ltx parse error: 4:1 broken");
  }

  #[test]
  fn places_a_dialect_diagnostic_at_the_file_it_blamed_before_the_declaring_config() {
    let blamed: LtxFinding = LtxFinding::from_diagnostic(
      "configs/system.ltx",
      &LtxResolutionDiagnostic {
        engine_behaviour: Some(String::from("silently dropped")),
        file: Some(String::from("configs/mod_system_patch.ltx")),
        message: String::from("Field 'cost' sits before any section and is ignored"),
        section: String::new(),
      },
      Some("configs/items/w_base.ltx"),
    );
    let declared: LtxFinding = LtxFinding::from_diagnostic(
      "configs/system.ltx",
      &LtxResolutionDiagnostic {
        engine_behaviour: None,
        file: None,
        message: String::from("Override patches a section nothing declares"),
        section: String::from("wpn_ak74"),
      },
      Some("configs/items/w_base.ltx"),
    );

    assert_eq!(blamed.file.as_deref(), Some("configs/mod_system_patch.ltx"));
    assert_eq!(blamed.section, None, "an empty section is the absence of one");
    assert_eq!(blamed.engine_behaviour.as_deref(), Some("silently dropped"));
    assert_eq!(declared.file.as_deref(), Some("configs/items/w_base.ltx"));
    assert_eq!(declared.section.as_deref(), Some("wpn_ak74"));
  }
}
