use glam::Vec3;
use xrf_error::XrfResult;

use crate::contract::viewport::render_level_hit::RenderLevelHit;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::view::scene_view::SceneView;
use crate::scene::static_scene::draw::slot::static_slot_info::StaticSlotInfo;

/// What a pick's texel says it met: a cluster, by its index and place, an impostor, by its index, or a helper's wire,
/// by the id its helper carries (`PICKED_HELPER` in `frame/helper_pick`).
const PICKED_CLUSTER: u32 = 1;
const PICKED_IMPOSTOR: u32 = 2;
const PICKED_HELPER: u32 = 3;

impl SceneView {
  /// Asks for a pick's texel back, its frame just submitted.
  pub fn request_pick(&self, slot: usize) {
    self.features.pick.request(slot);
  }

  /// A pick's texel once it is back; a view made since the pick has no readback for it, and it met nothing still drawn.
  pub fn take_pick(&self, slot: usize) -> Option<XrfResult<[u32; 4]>> {
    self.features.pick.take(slot)
  }

  /// Names what a pick's texel met, `unproject` turning its stored depth into where the ray met it and how many metres
  /// ahead of the camera, along its view, that point stands.
  pub fn resolve_pick(
    &self,
    scene: &LevelScene,
    [kind, cluster, place, stored]: [u32; 4],
    unproject: impl Fn(f32) -> (Vec3, f32),
  ) -> Option<RenderLevelHit> {
    let (point, depth): (Vec3, f32) = unproject(f32::from_bits(stored));
    let point: [f32; 3] = point.to_array();

    match kind {
      PICKED_CLUSTER => {}
      PICKED_HELPER => {
        return scene.helpers.find(cluster).map(|object| RenderLevelHit::Helper {
          object: object.id,
          point,
          depth,
        });
      }
      PICKED_IMPOSTOR => {
        return scene
          .statics
          .resolve_impostor_pick(cluster)
          .map(|(sector, shader_id)| RenderLevelHit::Surface {
            sector,
            shader_id: shader_id as u32,
            mesh: None,
            place: None,
            is_impostor: true,
            point,
            depth,
          });
      }
      _ => return None,
    }

    let (info, instance) = scene.statics.resolve_pick(cluster, place)?;

    if info.sector == StaticSlotInfo::NO_SECTOR {
      return scene
        .statics
        .resolve_spawn_pick(place)
        .map(|object| RenderLevelHit::Spawn { object, point, depth });
    }

    Some(RenderLevelHit::Surface {
      sector: info.sector,
      shader_id: info.shader_id as u32,
      mesh: info.mesh,
      place: instance,
      is_impostor: false,
      point,
      depth,
    })
  }
}
