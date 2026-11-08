use std::collections::VecDeque;

use glam::{Mat4, Vec3, Vec4};
use xrf_renderer_core::{ProxyHandle, ProxyStore};

use crate::camera::camera_frustum::CameraFrustum;
use crate::contract::report::frame::render_pool_use::RenderPoolUse;
use crate::contract::report::scene::render_static_report::RenderStaticReport;
use crate::frame::frame_encoder::FrameEncoder;
use crate::scene::static_scene::draw::static_cull_scratch::StaticCullScratch;
use crate::scene::static_scene::draw::static_sight::StaticSight;
use crate::scene::static_scene::geometry::static_geometry::StaticGeometry;
use crate::scene::static_scene::geometry::vertex::static_layout::StaticLayout;
use crate::scene::static_scene::item::model::static_model_proxy::StaticModelProxy;
use crate::scene::static_scene::item::object::static_object_proxy::StaticObjectProxy;
use crate::scene::static_scene::item::object::static_objects::StaticObjects;
use crate::scene::static_scene::item::static_sector_proxy::StaticSectorProxy;
use crate::scene::static_scene::item::static_sorted_place::StaticSortedPlace;
use crate::scene::static_scene::surface::static_surfaces::StaticSurfaces;
use crate::scene::static_scene::trace::static_trace_geometry::StaticTraceGeometry;
use crate::scene::static_scene::trace::static_trace_group::StaticTraceGroup;

mod static_scene_models;
mod static_scene_objects;
mod static_scene_picks;
mod static_scene_sectors;

/// Clusters a spawned model's row holds at most: the row cull walks a row's clusters on one thread.
const MODEL_ROW_CLUSTERS: usize = 32;

/// A level's static geometry on the GPU, as sectors, spawned models and the objects standing as them are added and
/// removed: the geometry's buffers each item takes ranges of, the surfaces every item shares, what a frame's culls
/// write into, and the items themselves, found by handle.
pub(crate) struct StaticScene {
  pub geometry: StaticGeometry,
  pub surfaces: StaticSurfaces,
  pub cull: StaticCullScratch,
  sectors: ProxyStore<StaticSectorProxy>,
  models: ProxyStore<StaticModelProxy>,
  objects: StaticObjects,
  /// The bounding sphere of every place that sways and its reach, which a light's shadow looks for in its range.
  swaying: Vec<(Vec4, f32)>,
  /// The farthest any swaying vertex reaches from its tree's foot, weighted by its rigidity and scaled by its place.
  sway_reach: f32,
  /// Items added and removed, which a view drawn with another count is drawn again for.
  contents: usize,
  /// The sphere in the world of each item added or removed lately, the first the change after `contents` was
  /// `changes_from`; none (a negative radius) for an item standing nowhere, a model alone.
  changes: VecDeque<Vec4>,
  changes_from: usize,
}

impl StaticScene {
  /// Changes the log keeps, past which a cache drawn before them all is drawn again whole.
  const CHANGES_KEPT: usize = 1024;

  /// Where an item standing nowhere of its own changes the scene: no sphere reaches it.
  const NOWHERE: Vec4 = Vec4::new(0.0, 0.0, 0.0, -1.0);

  pub(crate) fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
    Self {
      geometry: StaticGeometry::new(device, queue),
      surfaces: StaticSurfaces::new(device, queue),
      cull: StaticCullScratch::new(device),
      sectors: ProxyStore::new(),
      models: ProxyStore::new(),
      objects: StaticObjects::default(),
      swaying: Vec::new(),
      sway_reach: 0.0,
      contents: 0,
      changes: VecDeque::new(),
      changes_from: 0,
    }
  }

  /// How full the scene's pools are: what a report of its static draws starts from.
  pub(crate) fn get_pools(&self) -> RenderStaticReport {
    RenderStaticReport {
      surface_list: RenderPoolUse {
        used: 0,
        capacity: self.cull.get_list_capacity(),
      },
      ..self.geometry.get_pools()
    }
  }

  /// Stands a skinned object in a pose: each bone's matrix as three rows, from its bind to where it stands, this frame
  /// and the last, as many as its model has bones each; written with the scene's other changes.
  pub(crate) fn write_pose(
    &mut self,
    handle: ProxyHandle<StaticObjectProxy>,
    current: &[[Vec4; 3]],
    previous: &[[Vec4; 3]],
  ) {
    let Some((span, bones)) = self.objects.get(handle).and_then(|object| object.bones.as_ref()) else {
      return;
    };
    let count: usize = *bones as usize;
    let mut rows: Vec<[Vec4; 3]> = vec![[Vec4::X, Vec4::Y, Vec4::Z]; count * 2];

    for (to, from) in rows[..count].iter_mut().zip(current) {
      *to = *from;
    }

    for (to, from) in rows[count..].iter_mut().zip(previous) {
      *to = *from;
    }

    self.geometry.bones.write(span, 0, bytemuck::cast_slice(&rows));
  }

  /// Whether the device traces rays, so the vertices and indices are acceleration structures' input.
  pub(crate) fn is_traced(&self) -> bool {
    self.geometry.is_traced()
  }

  /// Every sector in the scene with its deferred baked sections and instanced meshes, which the level's acceleration
  /// structures take.
  pub(crate) fn list_trace_sectors(&self) -> impl Iterator<Item = (u32, &[StaticTraceGeometry], &[StaticTraceGroup])> {
    self.sectors.iter().map(|(_, sector)| {
      (
        sector.sector,
        sector.trace_geometries.as_slice(),
        sector.trace_groups.as_slice(),
      )
    })
  }

  /// Every spawned model the scene's objects stand as that the level's acceleration structures take, with its traced
  /// parts.
  pub(crate) fn list_trace_models(
    &self,
  ) -> impl Iterator<Item = (ProxyHandle<StaticModelProxy>, &[StaticTraceGeometry])> {
    self
      .models
      .iter()
      .filter(|(_, model)| !model.trace_geometries.is_empty())
      .map(|(handle, model)| (handle, model.trace_geometries.as_slice()))
  }

  /// Every spawned object standing as a traced model: its model, its place and its transform.
  pub(crate) fn list_trace_objects(&self) -> impl Iterator<Item = (ProxyHandle<StaticModelProxy>, u32, &Mat4)> {
    self
      .objects
      .iter()
      .filter(|(_, object)| {
        self
          .models
          .get(object.model)
          .is_some_and(|model| !model.trace_geometries.is_empty())
      })
      .map(|(_, object)| (object.model, object.place.get_offset(), &object.transform))
  }

  /// Items added and removed so far: what a cache notes it was drawn with, and asks `is_changed_since` about.
  pub(crate) fn get_contents(&self) -> usize {
    self.contents
  }

  /// Whether an item added or removed since the scene held `since` changes stands where `touches` says a sphere in the
  /// world reaches what a cache holds; anything did where the log no longer reaches back that far.
  pub(crate) fn is_changed_since(&self, since: usize, touches: impl Fn(Vec4) -> bool) -> bool {
    if since >= self.contents {
      return false;
    }

    if since < self.changes_from {
      return true;
    }

    self
      .changes
      .iter()
      .skip(since - self.changes_from)
      .any(|sphere| sphere.w >= 0.0 && touches(*sphere))
  }

  /// Counts a change of what the scene holds, at the sphere of the item added or removed.
  fn note_change(&mut self, sphere: Vec4) {
    self.contents += 1;
    self.changes.push_back(sphere);

    if self.changes.len() > Self::CHANGES_KEPT {
      self.changes.pop_front();
      self.changes_from += 1;
    }
  }

  /// Uploads what was added, changed and removed since the last frame, and readies the culls' buffers for the frame.
  pub(crate) fn prepare_draws(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut FrameEncoder) {
    self.geometry.flush(device, queue);
    self.cull.prepare(encoder);
  }

  /// The bounding sphere of every place that sways, and how far it reaches from its foot, weighted by its rigidity.
  pub(crate) fn list_swaying(&self) -> &[(Vec4, f32)] {
    &self.swaying
  }

  /// How far along `direction`, normalized, the level's sectors stand: the nearest and the farthest any reaches, none
  /// while it holds no sector.
  pub(crate) fn get_span(&self, direction: Vec3) -> Option<(f32, f32)> {
    self
      .sectors
      .as_slice()
      .iter()
      .map(|sector| {
        let along: f32 = sector.sphere.truncate().dot(direction);

        (along - sector.sphere.w, along + sector.sphere.w)
      })
      .reduce(|(nearest, farthest), (near, far)| (nearest.min(near), farthest.max(far)))
  }

  /// What the wind's amplitude is multiplied by to give the farthest any tree leans, in metres.
  pub(crate) fn get_sway_reach(&self) -> f32 {
    self.sway_reach
  }

  /// What of the scene drawn by passes of its own may be seen through a view's `frustum`.
  pub(crate) fn get_sight(&self, frustum: &CameraFrustum) -> StaticSight {
    let mut seen: StaticSight = StaticSight::default();

    for (sphere, sight) in self.list_sights() {
      if (seen | sight) != seen && frustum.reaches(*sphere) {
        seen |= sight;

        if seen == StaticSight::ALL {
          break;
        }
      }
    }

    seen
  }

  /// The spheres about the sectors' slots and the objects that bring what `is_wanted` asks for into sight, of those that
  /// stand within a view's `frustum`.
  pub(crate) fn list_seen_spheres<'s>(
    &'s self,
    frustum: &'s CameraFrustum,
    is_wanted: impl Fn(StaticSight) -> bool + 's,
  ) -> impl Iterator<Item = Vec4> + 's {
    self
      .list_sights()
      .filter(move |(sphere, sight)| is_wanted(*sight) && frustum.reaches(**sphere))
      .map(|(sphere, _)| *sphere)
  }

  /// The sectors' slots and the objects drawn by passes of their own, each with the sphere about it.
  fn list_sights(&self) -> impl Iterator<Item = (&Vec4, StaticSight)> {
    let sectors = self
      .sectors
      .as_slice()
      .iter()
      .flat_map(|sector| sector.sights.iter())
      .map(|(sphere, sight)| (sphere, *sight));
    let objects = self.objects.as_slice().iter().filter_map(|object| {
      let model: &StaticModelProxy = self.models.get(object.model)?;

      (!model.sight.is_empty()).then_some((&object.sphere, model.sight))
    });

    sectors.chain(objects)
  }

  /// The spawned objects with composited surfaces and the level's strictly sorted composited runs, which a view draws
  /// back to front itself.
  pub(crate) fn list_sorted_places(&self) -> impl Iterator<Item = StaticSortedPlace<'_>> {
    let objects = self.objects.as_slice().iter().filter_map(|object| {
      let model: &StaticModelProxy = self.models.get(object.model)?;

      (!model.sorted.is_empty()).then(|| StaticSortedPlace {
        sphere: object.sphere,
        place: object.place.get_offset(),
        layout: StaticLayout::Model,
        clusters: &model.sorted,
      })
    });
    let runs = self
      .sectors
      .as_slice()
      .iter()
      .flat_map(|sector| sector.sorted_runs.iter())
      .map(|run| StaticSortedPlace {
        sphere: run.sphere,
        place: run.place.unwrap_or(self.geometry.get_origin()),
        layout: run.layout,
        clusters: std::slice::from_ref(&run.clusters),
      });

    objects.chain(runs)
  }
}
