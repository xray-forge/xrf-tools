use serde::Serialize;
use xrf_ltx_inspect::{LtxAnchoredFinding, LtxFileStructure, LtxFileText};

/// One config as the authored view renders it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigsDocument {
  pub text: LtxFileText,
  pub structure: LtxFileStructure,
  /// What is wrong with this file itself: it will not parse, or an `#include` reached nothing.
  pub findings: Vec<LtxAnchoredFinding>,
}
