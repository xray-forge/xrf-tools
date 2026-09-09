use serde::Serialize;
use xrf_ltx_inspect::{LtxResolvedDiagnostic, LtxResolvedSection};

/// What `ltx inspect` answers about one section.
///
/// The section carries its own entry point and per-field origins, so this adds only the two things that are about the
/// run rather than about the section: which dialect resolved it, and what that dialect wanted said about it. The
/// diagnostics are narrowed to this section, because a whole tree's warnings are `ltx verify`'s answer and not this
/// command's.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxInspectReport {
  /// How the dialect that resolved this section names itself.
  pub dialect: String,
  pub section: LtxResolvedSection,
  pub diagnostics: Vec<LtxResolvedDiagnostic>,
}
