use std::sync::Arc;

use tauri::State;

use crate::core::jobs::{JobDescription, JobRegistry};

/// Report every running job and the last few that finished.
///
/// Answers on demand rather than from stored snapshots: a running job's progress is read off its own handle when
/// somebody actually looks, so nothing has to be pushed anywhere for this to be current.
///
/// A running job's progress here can lag what its own channel has already delivered by up to one emission interval.
/// That is why this is for a listing and for re-attaching, and never merged into a view that is already receiving
/// updates for the job it is watching.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "list"))]
#[tauri::command(rename = "list")]
pub fn jobs_list(registry: State<'_, Arc<JobRegistry>>) -> Vec<JobDescription> {
  registry.list()
}
