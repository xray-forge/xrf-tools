use xrf_level::LevelDetailsFile;

use crate::detail::detail_slot_counts::DetailSlotCounts;

/// A level's details lit: what xrDO_Light writes as `level.details`, how its slots were met, and the GPU that traced
/// its shadow rays, none where the CPU traced them.
#[derive(Clone, Debug, PartialEq)]
pub struct DetailLighting {
  /// `build.details` with every slot's bare corners cleared and every lit slot's light set.
  pub details: LevelDetailsFile,
  pub counts: DetailSlotCounts,
  pub gpu: Option<String>,
}
