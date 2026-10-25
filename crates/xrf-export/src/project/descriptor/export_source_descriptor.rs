use crate::parse::ExternSourceLocation;

/// Project-relative source location of an extern declaration.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSourceDescriptor {
  pub path: String,
  pub line: usize,
  pub column: usize,
  /// Last line of the declaration, inclusive, so its body can be fetched without parsing again.
  pub end_line: usize,
}

impl From<ExternSourceLocation> for ExportSourceDescriptor {
  fn from(location: ExternSourceLocation) -> Self {
    Self {
      path: location.path,
      line: location.line,
      column: location.column,
      end_line: location.end_line,
    }
  }
}
