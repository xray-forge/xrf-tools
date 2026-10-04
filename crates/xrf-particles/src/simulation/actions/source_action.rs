use glam::{Mat4, Vec3, Vec4};

use crate::data::actions::particle_action_source::ParticleActionSource;
use crate::simulation::actions::particle_action_step::ParticleActionStep;
use crate::simulation::particle::Particle;
use crate::simulation::particle_pool::ParticlePool;
use crate::simulation::particle_volume::ParticleVolume;

/// `PASource`: emits particles at a rate, each drawing its position, size, turn, velocity and colour from volumes.
pub(crate) struct SourceAction {
  flags: u32,
  position_local: ParticleVolume,
  velocity_local: ParticleVolume,
  position: ParticleVolume,
  velocity: ParticleVolume,
  rotation: ParticleVolume,
  size: ParticleVolume,
  color: ParticleVolume,
  alpha: f32,
  particle_rate: f32,
  age: f32,
  age_sigma: f32,
  parent_velocity: Vec3,
  parent_motion: f32,
}

impl SourceAction {
  /// `flSingleSize`: every axis takes the size drawn for x.
  const SINGLE_SIZE: u32 = 1 << 29;

  /// `flSilent`: emits nothing, which stopping an effect sets.
  const SILENT: u32 = 1 << 30;

  pub fn transform(&mut self, matrix: &Mat4, velocity: Vec3) {
    self.position.transform_from(&self.position_local, matrix);
    self.velocity.transform_direction_from(&self.velocity_local, matrix);
    self.parent_velocity = velocity * self.parent_motion;
  }

  /// `PlayEffect` and `StopEffect` clear and set `flSilent`.
  pub fn set_silent(&mut self, is_silent: bool) {
    if is_silent {
      self.flags |= Self::SILENT;
    } else {
      self.flags &= !Self::SILENT;
    }
  }

  pub fn execute(&self, pool: &mut ParticlePool, step: &ParticleActionStep) {
    if self.flags & Self::SILENT != 0 {
      return;
    }

    let wanted: f32 = self.particle_rate * step.dt;
    let mut rate: usize = wanted.floor() as usize;

    // The fraction of a particle this step is emitted by chance.
    if pool.get_random().next_fraction() < wanted - rate as f32 {
      rate += 1;
    }

    rate = rate.min(pool.get_max_particles().saturating_sub(pool.len()));

    for _ in 0..rate {
      let random = pool.get_random();
      let position: Vec3 = self.position.generate(random);
      let mut size: Vec3 = self.size.generate(random);

      if self.flags & Self::SINGLE_SIZE != 0 {
        size = Vec3::splat(size.x);
      }

      let rotation: Vec3 = self.rotation.generate(random);
      let velocity: Vec3 = self.velocity.generate(random) + self.parent_velocity;
      let color: Vec3 = self.color.generate(random);
      let age: f32 = self.age + random.next_normal(self.age_sigma);

      // Without `flVertexB_tracks` the engine passes an uninitialised previous position, which the first move
      // overwrites; it starts where the particle does here.
      pool.add(
        position,
        position,
        size,
        rotation,
        velocity,
        Particle::quantize_color(Vec4::new(color.x, color.y, color.z, self.alpha)),
        age,
      );
    }
  }
}

impl From<&ParticleActionSource> for SourceAction {
  fn from(action: &ParticleActionSource) -> Self {
    let position: ParticleVolume = ParticleVolume::from(&action.position);
    let velocity: ParticleVolume = ParticleVolume::from(&action.velocity);

    Self {
      flags: action.action_flags,
      position_local: position.clone(),
      velocity_local: velocity.clone(),
      position,
      velocity,
      rotation: ParticleVolume::from(&action.rot),
      size: ParticleVolume::from(&action.size),
      color: ParticleVolume::from(&action.color),
      alpha: action.alpha,
      particle_rate: action.particle_rate,
      age: action.age,
      age_sigma: action.age_sigma,
      parent_velocity: Vec3::from_array(action.parent_vel.to_array()),
      parent_motion: action.parent_motion,
    }
  }
}

#[cfg(test)]
mod tests {
  use glam::{Mat4, Vec3, Vec4};

  use super::SourceAction;
  use crate::simulation::actions::particle_action_step::ParticleActionStep;
  use crate::simulation::particle_engine_rules::ParticleEngineRules;
  use crate::simulation::particle_pool::ParticlePool;
  use crate::simulation::particle_volume::ParticleVolume;

  fn point(at: Vec3) -> ParticleVolume {
    ParticleVolume {
      kind: ParticleVolume::POINT,
      p1: at,
      p2: Vec3::ZERO,
      u: Vec3::ZERO,
      v: Vec3::ZERO,
      radius1: 0.0,
      radius2: 0.0,
      radius1_sqr: 0.0,
      radius2_sqr: 0.0,
    }
  }

  fn source(rate: f32) -> SourceAction {
    SourceAction {
      flags: SourceAction::SINGLE_SIZE,
      position_local: point(Vec3::new(1.0, 2.0, 3.0)),
      velocity_local: point(Vec3::Y),
      position: point(Vec3::new(1.0, 2.0, 3.0)),
      velocity: point(Vec3::Y),
      rotation: point(Vec3::new(0.25, 9.0, 9.0)),
      size: point(Vec3::new(0.5, 7.0, 7.0)),
      color: point(Vec3::new(1.0, 0.5, 0.0)),
      alpha: 0.5,
      particle_rate: rate,
      age: 0.0,
      age_sigma: 0.0,
      parent_velocity: Vec3::ZERO,
      parent_motion: 0.5,
    }
  }

  #[test]
  fn emits_the_whole_rate_and_dithers_the_fraction() {
    let rules: ParticleEngineRules = ParticleEngineRules::default();
    let mut pool: ParticlePool = ParticlePool::new(10, 1);

    // 2.5 particles this step; the first draw, 41 of 32767, is under the half, so three.
    source(5.0).execute(&mut pool, &ParticleActionStep::new(0.5, &rules));

    assert_eq!(pool.len(), 3);

    let particle = pool.get_particles()[0];

    assert_eq!(particle.position, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(particle.previous_position, particle.position);
    assert_eq!(particle.size, Vec3::splat(0.5));
    assert_eq!(particle.rotation, 0.25);
    assert_eq!(particle.velocity, Vec3::Y);
    assert_eq!(particle.color, Vec4::new(1.0, 127.0 / 255.0, 0.0, 127.0 / 255.0));
  }

  #[test]
  fn emits_no_more_than_the_effect_holds_and_nothing_while_silent() {
    let rules: ParticleEngineRules = ParticleEngineRules::default();
    let mut pool: ParticlePool = ParticlePool::new(2, 1);
    let mut action: SourceAction = source(100.0);

    action.execute(&mut pool, &ParticleActionStep::new(1.0, &rules));
    assert_eq!(pool.len(), 2);

    pool.clear();
    action.set_silent(true);
    action.execute(&mut pool, &ParticleActionStep::new(1.0, &rules));
    assert!(pool.is_empty());
  }

  #[test]
  fn inherits_the_parents_velocity_by_its_motion_share() {
    let mut action: SourceAction = source(1.0);

    action.transform(&Mat4::from_translation(Vec3::X), Vec3::new(4.0, 0.0, 0.0));

    assert_eq!(action.parent_velocity, Vec3::new(2.0, 0.0, 0.0));
    assert_eq!(action.position.p1, Vec3::new(2.0, 2.0, 3.0));
    assert_eq!(action.velocity.p1, Vec3::Y);
  }
}
