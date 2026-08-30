use serde::Serialize;

/// How a job stopped, carried by whatever result the operation reports.
///
/// A cancelled run answers with a result rather than an error: it wrote files, and a caller that is only told "stopped"
/// cannot say which ones. Each operation puts this on its own result type instead of the type becoming a sum, so the
/// callers that can never be cancelled — the CLI among them — are not forced through a discriminant they gain nothing
/// from. The trade is that a `Cancelled` result carries the same fields as a completed one, so its counts describe what
/// happened before the stop and nothing downstream may read them as a finished total.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum JobOutcome {
  /// The operation ran to the end of its work.
  #[default]
  Completed,
  /// The operation stopped at a safe boundary because it was asked to.
  Cancelled,
}
