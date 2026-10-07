use glam::{Mat4, Vec3, Vec4};
use xrf_renderer_core::ProxyHandle;
use xrf_visual::{LightsDescription, SectorPackage};

use crate::host::render_particle_definitions::RenderParticleDefinitions;
use crate::scene::level::particle_emitter_proxy::ParticleEmitterProxy;
use crate::scene::level::placed_effect::PlacedEffect;
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
  /// What the level's particles are made from, once read.
  AddParticles(RenderParticleDefinitions),
  /// A particle emitter, where it stands and the seed its effects' runs start from, nothing playing at it yet.
  AddEmitter {
    handle: ProxyHandle<ParticleEmitterProxy>,
    transform: Mat4,
    seed: i32,
  },
  /// Takes an emitter out with whatever plays at it.
  RemoveEmitter(ProxyHandle<ParticleEmitterProxy>),
  /// Plays an effect at an emitter unless one plays in that slot already.
  PlayEffect {
    handle: ProxyHandle<ParticleEmitterProxy>,
    effect: PlacedEffect,
    name: String,
  },
  /// Stops an effect: deferred, its particles live on; otherwise they go with it.
  StopEffect {
    handle: ProxyHandle<ParticleEmitterProxy>,
    effect: PlacedEffect,
    is_deferred: bool,
  },
  /// Moves an emitter and every effect at it, their sources taking on its velocity.
  MoveEmitter {
    handle: ProxyHandle<ParticleEmitterProxy>,
    transform: Mat4,
    velocity: Vec3,
  },
  /// Moves one effect's sources at a velocity where its emitter stands, as a campfire's wind carries its idle ones.
  CarryEffect {
    handle: ProxyHandle<ParticleEmitterProxy>,
    effect: PlacedEffect,
    velocity: Vec3,
  },
  /// A skinned object's bone matrices this frame, three rows a bone, and the last frame's beside them.
  PoseObject {
    handle: ProxyHandle<StaticObjectProxy>,
    current: Vec<[Vec4; 3]>,
    previous: Vec<[Vec4; 3]>,
  },
}
