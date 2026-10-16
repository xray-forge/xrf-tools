use glam::Vec4;

use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_cluster::StaticCluster;
use crate::scene::static_scene::static_geometry_base::StaticGeometryBase;
use crate::scene::static_scene::static_impostor::StaticImpostor;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_place::StaticPlace;
use crate::scene::static_scene::static_row::StaticRow;
use crate::scene::static_scene::static_sight::StaticSight;
use crate::scene::static_scene::static_slot::StaticSlot;
use crate::scene::static_scene::static_slot_info::StaticSlotInfo;
use crate::scene::static_scene::static_sorted_run::StaticSortedRun;
use crate::scene::static_scene::static_spans::StaticSpans;
use crate::scene::static_scene::static_surface::StaticSurface;
use crate::scene::static_scene::static_trace_geometry::StaticTraceGeometry;
use crate::scene::static_scene::static_trace_group::StaticTraceGroup;

/// What one scene item adds, gathered before its ranges are taken: its records name each other by their indices in
/// here, and `relocate` moves them to where its ranges put them, so each buffer gets one range an item.
#[derive(Default)]
pub struct StaticSceneWriter {
  pub words: [Vec<u32>; StaticLayout::COUNT],
  pub indices: Vec<u32>,
  pub clusters: Vec<StaticCluster>,
  /// Each cluster's layout, whose words its vertices are counted in.
  cluster_layouts: Vec<StaticLayout>,
  pub spheres: Vec<Vec4>,
  pub slots: Vec<StaticSlot>,
  pub places: Vec<StaticPlace>,
  pub rows: Vec<StaticRow>,
  /// Surfaces first asked for by this item, which every item shares once they are in.
  pub surfaces: Vec<StaticSurface>,
  pub impostors: Vec<StaticImpostor>,
  /// `StaticGeometry::CORNER_VECTORS` an impostor.
  pub corners: Vec<Vec4>,
  /// Two words a vertex of a skinned model.
  pub skins: Vec<u32>,
  pub slot_infos: Vec<StaticSlotInfo>,
  pub cluster_slots: Vec<u32>,
  pub impostor_shaders: Vec<u16>,
  /// Its deferred baked sections and instanced meshes, which the level's acceleration structures take.
  pub trace_geometries: Vec<StaticTraceGeometry>,
  pub trace_groups: Vec<StaticTraceGroup>,
  /// Its strictly sorted composited runs, which a view draws back to front.
  pub sorted_runs: Vec<StaticSortedRun>,
  /// The bounding sphere in the world of each of its sections and placed meshes drawn by a pass of its own, and what
  /// of them it brings into sight.
  pub sights: Vec<(Vec4, StaticSight)>,
  pub capacities: [u32; StaticBatch::LIST_COUNT],
}

impl StaticSceneWriter {
  /// Appends a geometry's vertex words to its layout's and its indices to the shared ones.
  pub fn put_words(&mut self, layout: StaticLayout, words: &[u32], indices: &[u32]) -> StaticGeometryBase {
    let base: StaticGeometryBase = StaticGeometryBase {
      vertex_start: (self.words[layout.get_index()].len() / StaticLayout::STRIDE as usize) as u32,
      index_start: self.indices.len() as u32,
    };

    self.words[layout.get_index()].extend_from_slice(words);
    self.indices.extend_from_slice(indices);

    base
  }

  /// Puts the clusters covering one index range as one slot; answers its index and how many clusters it took.
  pub fn put_slot(
    &mut self,
    base: &StaticGeometryBase,
    clusters: Vec<(u32, u32, Vec4)>,
    (kind, batch, surface, passes): (u32, StaticBatch, u32, u32),
    info: StaticSlotInfo,
  ) -> (u32, u32) {
    let slot: u32 = self.slots.len() as u32;
    let first_cluster: u32 = self.clusters.len() as u32;

    for (first_index, triangles, sphere) in clusters {
      self.clusters.push(StaticCluster {
        first_index: base.index_start + first_index,
        triangles,
        vertex_start: base.vertex_start,
        slot,
      });
      self.cluster_layouts.push(batch.layout);
      self.spheres.push(sphere);
      self.cluster_slots.push(slot);
    }

    let count: u32 = self.clusters.len() as u32 - first_cluster;

    self.slots.push(StaticSlot {
      first_cluster,
      cluster_count: count,
      place: 0,
      kind,
      batch: batch.get_index(),
      surface,
      passes,
      pad: 0,
    });
    self.slot_infos.push(info);

    (slot, count)
  }

  /// Moves every record's references from indices in here to where the item's ranges put them; the CPU's cluster slots
  /// stay counted from the item's own first.
  pub fn relocate(&mut self, spans: &StaticSpans) {
    let words: [u32; StaticLayout::COUNT] = std::array::from_fn(|layout| StaticSpans::get_offset(&spans.words[layout]));
    let indices: u32 = StaticSpans::get_offset(&spans.indices);
    let clusters: u32 = StaticSpans::get_offset(&spans.clusters);
    let slots: u32 = StaticSpans::get_offset(&spans.slots);
    let places: u32 = StaticSpans::get_offset(&spans.places);
    let impostors: u32 = StaticSpans::get_offset(&spans.impostors);

    for (cluster, layout) in self.clusters.iter_mut().zip(&self.cluster_layouts) {
      cluster.first_index += indices;
      cluster.vertex_start += words[layout.get_index()];
      cluster.slot += slots;
    }

    for slot in &mut self.slots {
      slot.first_cluster += clusters;
    }

    // An instance's impostor, as its place names it for the draw: -1 for none.
    for place in &mut self.places {
      if place.info.z >= 0.0 {
        place.info.z = (place.info.z as u32 + impostors) as f32;
      }
    }

    for row in &mut self.rows {
      row.place += places;
      row.slot += slots;

      if row.lod != StaticRow::NO_LOD {
        row.lod += impostors;
      }
    }

    for info in &mut self.slot_infos {
      info.first_place += places;
    }

    for geometry in self
      .trace_geometries
      .iter_mut()
      .chain(self.trace_groups.iter_mut().map(|group| &mut group.geometry))
    {
      geometry.first_index += indices;
      geometry.vertex_start += words[geometry.layout.get_index()];
    }

    for group in &mut self.trace_groups {
      group.first_place += places;
    }

    for run in &mut self.sorted_runs {
      run.clusters.0 += clusters;
      run.place = run.place.map(|place| place + places);
    }
  }
}
