use std::sync::Arc;

use tauri::State;

use crate::core::jobs::{JobDescription, JobRegistry};

/// Report every running job and the last few that finished.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "list"))]
#[tauri::command(rename = "list")]
pub fn jobs_list(registry: State<'_, Arc<JobRegistry>>) -> Vec<JobDescription> {
  registry.list()
}
