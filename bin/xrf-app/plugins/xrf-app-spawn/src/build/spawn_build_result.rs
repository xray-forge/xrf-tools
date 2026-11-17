use serde::Serialize;
use xrf_job::JobOutcome;
use xrf_spawn_builder::SpawnRootOutcome;

/// How a root spawn's build ended: what it wrote where it finished, nothing where it was stopped first.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SpawnBuildResult {
  pub(crate) outcome: JobOutcome,
  pub(crate) spawn: Option<SpawnRootOutcome>,
}
