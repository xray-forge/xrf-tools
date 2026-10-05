use glam::Vec4;
use xrf_renderer_core::ProxyHandle;
use xrf_visual::{LightsDescription, SectorPackage};

use crate::scene::static_scene::static_model::StaticModel;
use crate::scene::static_scene::static_model_place::StaticModelPlace;
use crate::scene::static_scene::static_model_proxy::StaticModelProxy;
use crate::scene::static_scene::static_object_proxy::StaticObjectProxy;
use crate::scene::static_scene::static_sector_proxy::StaticSectorProxy;

/// One change the world posts to a level's scene, applied at the start of the frame it arrives with; an add carries
/// the handle the world allocated for it, which its later updates and its remove name.
pub enum RenderSceneUpdate {
  AddSector {
    handle: ProxyHandle<StaticSectorProxy>,
    package: SectorPackage,
  },
  RemoveSector(ProxyHandle<StaticSectorProxy>),
  AddModel {
    handle: ProxyHandle<StaticModelProxy>,
    model: StaticModel,
  },
  /// Takes a model out with every object still standing as it.
  RemoveModel(ProxyHandle<StaticModelProxy>),
  AddObject {
    handle: ProxyHandle<StaticObjectProxy>,
    model: ProxyHandle<StaticModelProxy>,
    place: StaticModelPlace,
  },
  RemoveObject(ProxyHandle<StaticObjectProxy>),
  /// The level's lights, once read.
  AddLights(LightsDescription),
  /// A skinned object's bone matrices this frame, three rows a bone, and the last frame's beside them.
  PoseObject {
    handle: ProxyHandle<StaticObjectProxy>,
    current: Vec<[Vec4; 3]>,
    previous: Vec<[Vec4; 3]>,
  },
}
