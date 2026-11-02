use std::path::PathBuf;

use serde::Serialize;
use xrf_job::JobOutcome;

use crate::SpawnConversion;

/// What reached disk; cancellation is accepted only before writing begins.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct SpawnConversionResult {
  pub operation: SpawnConversion,
  pub destination: PathBuf,
  pub outcome: JobOutcome,
}
