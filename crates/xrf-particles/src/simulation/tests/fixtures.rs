use xrf_math::Vector3d;

use crate::chunks::particles_effects_chunk::ParticlesEffectsChunk;
use crate::chunks::particles_groups_chunk::ParticlesGroupsChunk;
use crate::chunks::particles_header_chunk::ParticlesHeaderChunk;
use crate::data::actions::particle_action_kill_old::ParticleActionKillOld;
use crate::data::actions::particle_action_move::ParticleActionMove;
use crate::data::actions::particle_action_source::ParticleActionSource;
use crate::data::particle_action::ParticleAction;
use crate::data::particle_action_type::ParticleActionType;
use crate::data::particle_domain::ParticleDomain;
use crate::data::particle_effect::ParticleEffect;
use crate::data::particle_effect_sprite::ParticleEffectSprite;
use crate::data::particle_group::ParticleGroup;
use crate::data::particle_group_effect::ParticleGroupEffect;
use crate::particles_file::ParticlesFile;
use crate::simulation::particle_library::ParticleLibrary;

/// A point domain at a position.
pub fn point(x: f32, y: f32, z: f32) -> ParticleDomain {
  ParticleDomain {
    domain_type: 0,
    coordinates: (Vector3d::new(x, y, z), Vector3d::new(0.0, 0.0, 0.0)),
    basis: (Vector3d::new(0.0, 0.0, 0.0), Vector3d::new(0.0, 0.0, 0.0)),
    radius1: 0.0,
    radius2: 0.0,
    radius1_sqr: 0.0,
    radius2_sqr: 0.0,
  }
}

/// A source at the origin emitting at a rate, still, unit sized and white.
pub fn source(rate: f32) -> ParticleAction {
  ParticleAction::Source(Box::new(ParticleActionSource {
    action_flags: 0,
    action_type: ParticleActionType::Source,
    position: point(0.0, 0.0, 0.0),
    velocity: point(0.0, 1.0, 0.0),
    rot: point(0.0, 0.0, 0.0),
    size: point(1.0, 1.0, 1.0),
    color: point(1.0, 1.0, 1.0),
    alpha: 1.0,
    particle_rate: rate,
    age: 0.0,
    age_sigma: 0.0,
    parent_vel: Vector3d::new(0.0, 0.0, 0.0),
    parent_motion: 0.0,
  }))
}

pub fn moving() -> ParticleAction {
  ParticleAction::Move(Box::new(ParticleActionMove {
    action_flags: 0,
    action_type: ParticleActionType::Move,
  }))
}

pub fn kill_old(age_limit: f32) -> ParticleAction {
  ParticleAction::KillOld(Box::new(ParticleActionKillOld {
    action_flags: 0,
    action_type: ParticleActionType::KillOld,
    age_limit,
    kill_less_than: 0,
  }))
}

/// A sprite effect of actions, with flags and a time limit.
pub fn effect(name: &str, max_particles: u32, actions: Vec<ParticleAction>) -> ParticleEffect {
  ParticleEffect {
    version: 1,
    name: String::from(name),
    max_particles,
    actions,
    flags: 1,
    frame: None,
    sprite: ParticleEffectSprite {
      shader_name: String::from("particles\\add"),
      texture_name: String::from("pfx\\pfx_flame"),
    },
    time_limit: None,
    collision: None,
    velocity_scale: None,
    description: None,
    rotation: None,
    editor_data: None,
  }
}

/// A group effect played from `time_0` to `time_1` with flags.
pub fn group_effect(name: &str, time_0: f32, time_1: f32, flags: u32) -> ParticleGroupEffect {
  ParticleGroupEffect {
    name: String::from(name),
    on_play_child_name: String::new(),
    on_birth_child_name: String::new(),
    on_dead_child_name: String::new(),
    time_0,
    time_1,
    flags,
  }
}

pub fn group(name: &str, time_limit: f32, effects: Vec<ParticleGroupEffect>) -> ParticleGroup {
  ParticleGroup {
    version: 3,
    name: String::from(name),
    flags: 0,
    time_limit,
    effects,
    description: None,
    effects_old: None,
  }
}

pub fn library(effects: Vec<ParticleEffect>, groups: Vec<ParticleGroup>) -> ParticleLibrary {
  ParticleLibrary::from(ParticlesFile {
    header: ParticlesHeaderChunk { version: 1 },
    effects: ParticlesEffectsChunk { effects },
    groups: ParticlesGroupsChunk { groups },
  })
}
