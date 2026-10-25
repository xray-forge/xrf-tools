use serde::Serialize;

/// One recipe field the descriptor asks for that this build does not carry out.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureBuildOmissionReport {
  /// The descriptor field this sits beside, under the name the SDK gives it.
  pub field: String,
  /// Why the build does not do it, in words a person deciding whether to rebuild can act on.
  pub reason: String,
}
