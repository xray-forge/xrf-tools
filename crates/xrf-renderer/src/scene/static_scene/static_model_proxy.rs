use glam::{Vec3, Vec4};

use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_slot_info::StaticSlotInfo;
use crate::scene::static_scene::static_spans::StaticSpans;

/// One spawned model as the scene holds it, once whatever objects stand as it: the ranges its geometry, clusters, slots
/// and skin take, and what an object standing as it needs.
#[derive(Debug)]
pub struct StaticModelProxy {
  pub spans: StaticSpans,
  /// Each slot it is cut into, its clusters and its batch: a row an object stands as each.
  pub slots: Vec<(u32, u32, StaticBatch)>,
  /// The runs of clusters its composited parts are cut into, as first cluster and count, in their parts' order.
  pub sorted: Vec<(u32, u32)>,
  /// Each of its slots' part, and each of its clusters' slot, both counted from its own first.
  pub slot_infos: Vec<StaticSlotInfo>,
  pub cluster_slots: Vec<u32>,
  /// Its bounding sphere and its box, its least and greatest corners, in its own space.
  pub sphere: Vec4,
  pub bounds: [Vec3; 2],
  /// Its first vertex in the models' arena, which its skin's links are counted from.
  pub vertex_start: u32,
  /// Its skin's first link and its skeleton's bones, where it is skinned.
  pub skin: Option<(u32, u32)>,
}
