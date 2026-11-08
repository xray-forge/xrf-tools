use glam::{Vec3, Vec4};

use crate::contract::overlay::render_selection_target::RenderSelectionTarget;
use crate::scene::static_scene::draw::slot::static_slot_info::StaticSlotInfo;
use crate::scene::static_scene::draw::static_selection::StaticSelection;
use crate::scene::static_scene::geometry::static_spans::StaticSpans;
use crate::scene::static_scene::item::object::static_object_box::StaticObjectBox;
use crate::scene::static_scene::item::object::static_object_proxy::StaticObjectProxy;
use crate::scene::static_scene::item::static_sector_proxy::StaticSectorProxy;
use crate::scene::static_scene::static_scene::StaticScene;

impl StaticScene {
  /// The sector and shader table entry of a picked impostor.
  pub fn resolve_impostor_pick(&self, impostor: u32) -> Option<(u32, u16)> {
    self
      .sectors
      .as_slice()
      .iter()
      .find(|sector| StaticSpans::contains(sector.spans.impostors.as_ref(), impostor))
      .and_then(|sector| {
        let local: u32 = impostor - StaticSpans::get_offset(sector.spans.impostors.as_ref());

        Some((sector.sector, *sector.impostor_shaders.get(local as usize)?))
      })
  }

  /// What a pick's cluster and place were part of: its slot's part, and the place's instance where it is a mesh's.
  pub fn resolve_pick(&self, cluster: u32, place: u32) -> Option<(StaticSlotInfo, Option<u32>)> {
    let (spans, slot_infos, cluster_slots): (&StaticSpans, &[StaticSlotInfo], &[u32]) = self
      .sectors
      .as_slice()
      .iter()
      .map(|sector| {
        (
          &sector.spans,
          sector.slot_infos.as_slice(),
          sector.cluster_slots.as_slice(),
        )
      })
      .chain(self.models.as_slice().iter().map(|model| {
        (
          &model.spans,
          model.slot_infos.as_slice(),
          model.cluster_slots.as_slice(),
        )
      }))
      .find(|(spans, _, _)| StaticSpans::contains(spans.clusters.as_ref(), cluster))?;
    let local: u32 = cluster - StaticSpans::get_offset(spans.clusters.as_ref());
    let info: StaticSlotInfo = *slot_infos.get(*cluster_slots.get(local as usize)? as usize)?;

    Some((info, info.mesh.map(|_| place.saturating_sub(info.first_place))))
  }

  /// The spawned object a picked place is, where it is one.
  pub fn resolve_spawn_pick(&self, place: u32) -> Option<u32> {
    self.objects.find_at(place).map(|object| object.object)
  }

  /// Whether a spawned object stands in the scene.
  pub fn has_object(&self, object: u32) -> bool {
    self.objects.contains(object)
  }

  /// A spawned object's bounding sphere in renderer space, while it stands in the scene.
  pub fn get_object_sphere(&self, object: u32) -> Option<Vec4> {
    self.objects.find(object).map(|object| object.sphere)
  }

  /// The place a spawned object stands at, which its drawn clusters carry, while it stands in the scene.
  pub fn get_object_place(&self, object: u32) -> Option<u32> {
    self.objects.find(object).map(|object| object.place.get_offset())
  }

  /// A spawned object's box as it stands, while it stands in the scene.
  pub fn get_object_box(&self, object: u32) -> Option<StaticObjectBox> {
    let object: &StaticObjectProxy = self.objects.find(object)?;
    let [min, max]: [Vec3; 2] = self.models.get(object.model)?.bounds;

    Some(StaticObjectBox {
      transform: object.transform,
      min,
      max,
    })
  }

  /// What a selection marks, once what it names is in the scene: a spawned object's place; a mesh's place; or the
  /// runs of clusters drawing a sector's baked geometry of one shader table entry.
  pub fn resolve_selection(&self, target: &RenderSelectionTarget) -> Option<StaticSelection> {
    match *target {
      RenderSelectionTarget::Spawn { object } => Some(StaticSelection {
        place: Some(self.objects.find(object)?.place.get_offset()),
        runs: vec![StaticSelection::ANY_CLUSTER],
      }),
      RenderSelectionTarget::Surface {
        sector,
        mesh: Some(mesh),
        place: Some(place),
        ..
      } => {
        let info: &StaticSlotInfo = self
          .find_sector(sector)?
          .slot_infos
          .iter()
          .find(|info| info.mesh == Some(mesh))?;

        Some(StaticSelection {
          place: Some(info.first_place + place),
          runs: vec![StaticSelection::ANY_CLUSTER],
        })
      }
      RenderSelectionTarget::Surface { sector, shader_id, .. } => {
        let proxy: &StaticSectorProxy = self.find_sector(sector)?;
        let first: u32 = StaticSpans::get_offset(proxy.spans.clusters.as_ref());
        let mut runs: Vec<[u32; 2]> = Vec::new();

        for (local, slot) in proxy.cluster_slots.iter().enumerate() {
          let Some(info) = proxy.slot_infos.get(*slot as usize) else {
            continue;
          };

          if u32::from(info.shader_id) != shader_id || info.mesh.is_some() {
            continue;
          }

          let cluster: u32 = first + local as u32;

          match runs.last_mut() {
            Some(run) if run[1] == cluster => run[1] = cluster + 1,
            _ => runs.push([cluster, cluster + 1]),
          }
        }

        (!runs.is_empty()).then_some(StaticSelection { place: None, runs })
      }
    }
  }
}
