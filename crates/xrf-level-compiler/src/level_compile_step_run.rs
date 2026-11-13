use serde::Serialize;

use crate::detail::detail_slot_counts::DetailSlotCounts;

/// What ran a step of a level's compile, and what it says of the run.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum LevelCompileStepRun {
  /// One of the SDK's compilers: its path, and its log where it wrote one.
  Program { program: String, log: Option<String> },
  /// The detail lighter ([`crate::DetailLighter`]), how it met the slots, and the GPU that traced its shadow rays,
  /// none where the CPU traced them.
  DetailLighter {
    slots: DetailSlotCounts,
    gpu: Option<String>,
  },
}
