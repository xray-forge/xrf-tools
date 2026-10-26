use serde::Serialize;

/// The `$scheme` a resolved section ends up carrying, and whether the project declares it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxStructureScheme {
  pub name: String,
  /// Whether a scheme file declares it. False is itself a finding, and the verifier reports it as one.
  pub is_declared: bool,
}
