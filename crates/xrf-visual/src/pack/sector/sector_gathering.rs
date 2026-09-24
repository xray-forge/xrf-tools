use std::collections::BTreeMap;

use crate::data::sector::sector_skip::SectorSkip;
use crate::pack::sector::sector_impostor_arrays::SectorImpostorArrays;
use crate::pack::sector::sector_instance_gathering::SectorInstanceGathering;
use crate::pack::sector::sector_instance_key::SectorInstanceKey;
use crate::pack::sector::sector_section_gathering::SectorSectionGathering;
use crate::pack::sector::sector_vertex_arrays::SectorVertexArrays;

/// Everything a walk of one sector gathered, for its buffer to be built from.
pub(crate) struct SectorGathering {
  pub(crate) arrays: SectorVertexArrays,
  pub(crate) sections: BTreeMap<u16, SectorSectionGathering>,
  pub(crate) instances: BTreeMap<SectorInstanceKey, SectorInstanceGathering>,
  pub(crate) impostors: SectorImpostorArrays,
  pub(crate) skipped: Vec<SectorSkip>,
}
