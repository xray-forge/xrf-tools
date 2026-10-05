use glam::Vec4;
use xrf_renderer_core::ProxyHandle;

use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_row::StaticRow;
use crate::scene::static_scene::static_slot_info::StaticSlotInfo;
use crate::scene::static_scene::static_spans::StaticSpans;

/// One sector as the scene holds it: the ranges its geometry, clusters, slots, places and impostors take, its rows, and
/// on the CPU what turns a pick of it back into names.
#[derive(Debug)]
pub struct StaticSectorProxy {
  /// The sector, by its index in the level.
  pub sector: u32,
  /// Bytes of its pack.
  pub bytes: u64,
  pub spans: StaticSpans,
  pub rows: Vec<ProxyHandle<StaticRow>>,
  /// Entries of the visible list it may need a batch.
  pub capacities: [u32; StaticBatch::COUNT],
  /// Each of its slots' part, and each of its clusters' slot, both counted from its own first.
  pub slot_infos: Vec<StaticSlotInfo>,
  pub cluster_slots: Vec<u32>,
  /// Each of its impostors' shader table entry, from its own first.
  pub impostor_shaders: Vec<u16>,
  /// The bounding sphere of each of its places that sways, and its reach.
  pub swaying: Vec<(Vec4, f32)>,
}
