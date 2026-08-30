use serde::Serialize;

/// How a job that is no longer running ended.
///
/// Wider than `xrf_job::JobOutcome` on purpose: that one is what an operation reports about its own work, and an
/// operation that failed reports nothing at all — the failure travels as the command's error. The registry watches
/// from outside and has to describe that case too, or a job that blew up would sit in the listing looking finished.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum JobConclusion {
  Completed,
  Cancelled,
  Failed,
}
