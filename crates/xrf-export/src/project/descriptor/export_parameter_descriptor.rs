use crate::manifest::ExternParameter;

/// One callable parameter projected for the application-facing exports project.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportParameterDescriptor {
  pub name: String,
  pub typing: String,
  pub description: Option<String>,
  pub is_optional: bool,
}

impl From<ExternParameter> for ExportParameterDescriptor {
  fn from(parameter: ExternParameter) -> Self {
    Self {
      is_optional: parameter.is_optional(),
      name: parameter.name,
      typing: parameter.type_name,
      description: parameter.doc,
    }
  }
}
