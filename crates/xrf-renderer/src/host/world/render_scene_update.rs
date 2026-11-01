use glam::{Mat4, Vec3, Vec4};
use xrf_renderer_core::ProxyHandle;
use xrf_visual::{LightsDescription, SectorPackage};

use crate::host::level::helpers::render_level_helpers::RenderLevelHelpers;
use crate::host::level::particles::placed_effect::PlacedEffect;
use crate::host::level::particles::render_particle_definitions::RenderParticleDefinitions;
use crate::host::level::render_level_details::RenderLevelDetails;
use crate::host::level::spawn::model::static_model_place::StaticModelPlace;
use crate::scene::level::particles::placed::particle_emitter_proxy::ParticleEmitterProxy;
use crate::scene::static_scene::item::model::static_model::StaticModel;
use crate::scene::static_scene::item::model::static_model_proxy::StaticModelProxy;
use crate::scene::static_scene::item::object::static_object_proxy::StaticObjectProxy;
use crate::scene::static_scene::item::static_sector_proxy::StaticSectorProxy;

/// One change the world posts to a level's scene, applied at the start of the frame it arrives with: an add carries
/// the handle the world allocated for it, which its later updates and its remove name; a set replaces the one part of
/// the level it names.
pub enum RenderSceneUpdate {
  AddSector {
    handle: ProxyHandle<StaticSectorProxy>,
    package: SectorPackage,
  },
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
  /// The level's lights, once read, in place of any read before.
  SetLights(LightsDescription),
  /// The level's grass, once read, in place of any read before; none for a level without a detail library.
  SetGrass(Option<RenderLevelDetails>),
  /// The level objects the viewer draws helpers for, once read, in place of any read before.
  SetHelpers(RenderLevelHelpers),
  /// What the level's particles are made from, once read, in place of any read before.
  SetParticles(RenderParticleDefinitions),
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
