use serde::Serialize;
use xrf_ltx::LtxResolutionDiagnostic;

/// Something the dialect wanted said about a root, short of refusing it.
///
/// The serializable mirror of [`LtxResolutionDiagnostic`], which the core intentionally does not carry a wire shape for.
/// No severity, because there is only one: everything the engine refuses to start on comes back as an error from the
/// resolve, so a diagnostic exists precisely where the game would say nothing.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxResolvedDiagnostic {
  pub section: String,
  /// Engine identity of the config the dialect blamed, where it named one.
  pub file: Option<String>,
  pub message: String,
  /// What the engine does with the same input, where that differs from reporting it.
  pub engine_behaviour: Option<String>,
}

impl From<&LtxResolutionDiagnostic> for LtxResolvedDiagnostic {
  fn from(diagnostic: &LtxResolutionDiagnostic) -> Self {
    Self {
      engine_behaviour: diagnostic.engine_behaviour.clone(),
      file: diagnostic.file.clone(),
      message: diagnostic.message.clone(),
      section: diagnostic.section.clone(),
    }
  }
}
