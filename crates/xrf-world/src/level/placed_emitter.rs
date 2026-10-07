use std::sync::Arc;

use glam::{Mat4, Vec3};
use xrf_renderer::{ParticleEmitterProxy, PlacedEffect, RenderParticleSource, RenderSceneUpdate};
use xrf_renderer_core::ProxyHandle;
use xrf_visual::ZoneSphere;

/// One placement of a level's particles: the emitter the scene plays its effects at, what it plays and where it stands
/// this frame, which effects the world has the scene play there, and for a campfire whether it was lit when last placed.
pub struct PlacedEmitter {
  pub handle: ProxyHandle<ParticleEmitterProxy>,
  /// What it plays, shared so a frame reads it while it posts.
  pub source: Arc<RenderParticleSource>,
  pub transform: Mat4,
  /// The object motion carrying its zone, which moves it each frame.
  pub motion: Option<String>,
  /// Its zone's sphere, offset from the transform's place, which Monolith measures how far the camera stands by.
  pub zone_sphere: Option<ZoneSphere>,
  pub campfire_lit: Option<bool>,
  /// Which effects the scene was told to play here and not told to stop since.
  played: [bool; PlacedEffect::COUNT],
}

impl PlacedEmitter {
  pub fn new(
    handle: ProxyHandle<ParticleEmitterProxy>,
    source: RenderParticleSource,
    transform: Mat4,
    (motion, zone_sphere): (Option<String>, Option<ZoneSphere>),
  ) -> Self {
    Self {
      handle,
      source: Arc::new(source),
      transform,
      motion,
      zone_sphere,
      campfire_lit: None,
      played: [false; PlacedEffect::COUNT],
    }
  }

  /// Has the scene play an effect here unless it already does, as a zone makes its idle object only once.
  pub fn play(&mut self, updates: &mut Vec<RenderSceneUpdate>, effect: PlacedEffect, name: &str) {
    if !std::mem::replace(&mut self.played[effect.get_index()], true) {
      updates.push(RenderSceneUpdate::PlayEffect {
        handle: self.handle,
        effect,
        name: name.to_owned(),
      });
    }
  }

  /// Has the scene stop an effect at once, its particles gone with it: `Stop(FALSE)`, then `Destroy`.
  pub fn stop(&mut self, updates: &mut Vec<RenderSceneUpdate>, effect: PlacedEffect) {
    if std::mem::replace(&mut self.played[effect.get_index()], false) {
      updates.push(RenderSceneUpdate::StopEffect {
        handle: self.handle,
        effect,
        is_deferred: false,
      });
    }
  }

  /// Has the scene move an effect's sources at a velocity where it stands, as `CZoneCampfire::shedule_Update` carries
  /// its idle particles by the wind.
  pub fn carry(&self, updates: &mut Vec<RenderSceneUpdate>, effect: PlacedEffect, velocity: Vec3) {
    if self.played[effect.get_index()] {
      updates.push(RenderSceneUpdate::CarryEffect {
        handle: self.handle,
        effect,
        velocity,
      });
    }
  }

  /// Moves it and everything playing at it (`CCustomZone::OnMove`, `UpdateParent`), their sources taking on its velocity.
  pub fn move_to(&mut self, updates: &mut Vec<RenderSceneUpdate>, transform: Mat4, velocity: Vec3) {
    self.transform = transform;
    updates.push(RenderSceneUpdate::MoveEmitter {
      handle: self.handle,
      transform,
      velocity,
    });
  }
}
