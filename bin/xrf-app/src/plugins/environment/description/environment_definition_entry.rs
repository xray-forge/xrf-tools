use serde::Serialize;
use xrf_environment::EnvironmentCatalog;

/// One definition of a catalog, a sun, a collection or an ambient, as a list shows it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentDefinitionEntry {
  pub name: String,
  pub file: String,
  /// Findings in its section.
  pub findings: u32,
}

impl EnvironmentDefinitionEntry {
  pub fn of(name: &str, file: &str, catalog: &EnvironmentCatalog) -> Self {
    Self {
      file: file.to_owned(),
      findings: catalog
        .findings
        .iter()
        .filter(|finding| finding.file == file && finding.section.as_deref() == Some(name))
        .count() as u32,
      name: name.to_owned(),
    }
  }
}
