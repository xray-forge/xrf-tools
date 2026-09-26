use crate::data::alife::alife_object::AlifeObject;
use crate::data::alife::spawn_skipped_object::SpawnSkippedObject;

/// The objects a spawn places on one level, and what of the whole spawn could not be read.
#[derive(Clone, Debug, PartialEq)]
pub struct SpawnLevelObjects {
  /// The objects standing on the level, in the spawn's order.
  pub objects: Vec<AlifeObject>,
  /// Objects the whole spawn declares, on every level.
  pub total: u32,
  /// Objects of the whole spawn that could not be read, on whichever level they stand, which cannot be told.
  pub skipped: Vec<SpawnSkippedObject>,
}
