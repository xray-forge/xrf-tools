//! A placement's emitter: the scene is told to play and stop its effects only as they change.

use glam::{Mat4, Vec3};
use xrf_renderer::{ParticleEmitterProxy, PlacedEffect, RenderParticleSource, RenderSceneUpdate};
use xrf_renderer_core::{ProxyAllocator, ProxyHandle};

use crate::level::placed_emitter::PlacedEmitter;

fn emitter() -> PlacedEmitter {
  let handle: ProxyHandle<ParticleEmitterProxy> = ProxyAllocator::new().allocate();

  PlacedEmitter::new(
    handle,
    RenderParticleSource::Static {
      name: String::from("fire"),
    },
    Mat4::IDENTITY,
    (None, None),
  )
}

#[test]
fn plays_and_stops_an_effect_once_however_often_asked() {
  let mut emitter: PlacedEmitter = emitter();
  let mut updates: Vec<RenderSceneUpdate> = Vec::new();

  emitter.stop(&mut updates, PlacedEffect::Idle);
  emitter.play(&mut updates, PlacedEffect::Idle, "fire");
  emitter.play(&mut updates, PlacedEffect::Idle, "fire");

  assert!(matches!(
    updates.as_slice(),
    [RenderSceneUpdate::PlayEffect {
      effect: PlacedEffect::Idle,
      ..
    }]
  ));

  updates.clear();
  emitter.stop(&mut updates, PlacedEffect::Idle);
  emitter.stop(&mut updates, PlacedEffect::Idle);

  assert!(matches!(
    updates.as_slice(),
    [RenderSceneUpdate::StopEffect {
      effect: PlacedEffect::Idle,
      is_deferred: false,
      ..
    }]
  ));
}

#[test]
fn carries_only_an_effect_it_plays() {
  let mut emitter: PlacedEmitter = emitter();
  let mut updates: Vec<RenderSceneUpdate> = Vec::new();

  emitter.carry(&mut updates, PlacedEffect::Idle, Vec3::X);
  assert!(updates.is_empty());

  emitter.play(&mut updates, PlacedEffect::Idle, "fire");
  updates.clear();
  emitter.carry(&mut updates, PlacedEffect::Idle, Vec3::X);
  assert!(matches!(updates.as_slice(), [RenderSceneUpdate::CarryEffect { .. }]));
}
