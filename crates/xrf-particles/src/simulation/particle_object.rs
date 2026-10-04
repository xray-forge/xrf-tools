use glam::{Mat4, Vec3};

use crate::simulation::particle_engine_rules::ParticleEngineRules;
use crate::simulation::particle_instance::ParticleInstance;
use crate::simulation::particle_library::ParticleLibrary;

/// `CParticlesObject`: a placed effect or group, updated while drawn and by the distance-paced scheduler.
pub struct ParticleObject {
  instance: ParticleInstance,
  /// `dwLastTime`: the milliseconds it was last updated at.
  last_time: u64,
  /// When the scheduler next updates it.
  next_scheduled: u64,
  /// `m_bLooped`: plays until stopped, its time limit not positive.
  is_looped: bool,
  /// `m_bStopping`.
  is_stopping: bool,
}

impl ParticleObject {
  /// `Play` backdates the last update by this, so the first update takes a step.
  const PLAY_BACKDATE: u64 = 33;

  /// The scheduler's interval bounds for a particle object: `max(30, t_min = 20)` and `(1000 + t_max = 50) / 2`.
  const SCHEDULE_MIN: u32 = 30;
  const SCHEDULE_MAX: u32 = 525;

  /// `shedule_Scale`'s distance unit: the interval reaches its maximum this far away.
  const SCHEDULE_DISTANCE: f32 = 200.0;

  pub fn new(instance: ParticleInstance) -> Self {
    Self {
      is_looped: instance.get_time_limit() <= 0.0,
      instance,
      last_time: 0,
      next_scheduled: 0,
      is_stopping: false,
    }
  }

  pub fn get_instance(&self) -> &ParticleInstance {
    &self.instance
  }

  pub fn is_looped(&self) -> bool {
    self.is_looped
  }

  pub fn is_playing(&self) -> bool {
    self.instance.is_playing()
  }

  pub fn is_stopping(&self) -> bool {
    self.is_stopping
  }

  /// `UpdateParent`: places the effect or group in world space.
  pub fn update_parent(&mut self, matrix: &Mat4, velocity: Vec3) {
    self.instance.update_parent(matrix, velocity);
  }

  /// `Play`: plays at once and takes a first update as if 33 ms had passed.
  pub fn play(&mut self, now: u64, library: &ParticleLibrary, rules: &ParticleEngineRules) {
    self.instance.play();
    self.last_time = now.saturating_sub(Self::PLAY_BACKDATE);
    self.perform(now, library, rules);
    self.is_stopping = false;
  }

  pub fn stop(&mut self, is_deferred: bool) {
    self.instance.stop(is_deferred);
    self.is_stopping = true;
  }

  /// The scheduler's update when due, then the drawn update when in view (`shedule_Update`, `renderable_Render`).
  pub fn advance(
    &mut self,
    now: u64,
    view: Vec3,
    is_in_view: bool,
    library: &ParticleLibrary,
    rules: &ParticleEngineRules,
  ) {
    if now >= self.next_scheduled {
      self.perform(now, library, rules);

      let (center, _) = self.instance.get_bounds().get_sphere();

      self.next_scheduled = now + Self::get_schedule_interval(view.distance(center)) as u64;
    }

    if is_in_view {
      self.perform(now, library, rules);
    }
  }

  /// `CSheduler`'s interval for an object at a distance from the view, in milliseconds.
  pub fn get_schedule_interval(distance: f32) -> u32 {
    let scale: f32 = distance / Self::SCHEDULE_DISTANCE;
    let interval: u32 = Self::SCHEDULE_MIN
      .saturating_add_signed((((Self::SCHEDULE_MAX - Self::SCHEDULE_MIN) as f32) * scale).floor() as i32);

    interval.clamp(Self::SCHEDULE_MIN, Self::SCHEDULE_MAX)
  }

  /// `PerformAllTheWork`: an update by the milliseconds since the last one, if any passed.
  fn perform(&mut self, now: u64, library: &ParticleLibrary, rules: &ParticleEngineRules) {
    let elapsed: u64 = now.saturating_sub(self.last_time);

    if elapsed > 0 {
      self
        .instance
        .update(elapsed.min(u32::MAX as u64) as u32, library, rules);
      self.last_time = now;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::ParticleObject;

  #[test]
  fn schedules_near_objects_often_and_far_ones_at_the_cap() {
    assert_eq!(ParticleObject::get_schedule_interval(0.0), 30);
    // 30 + floor(495 * 0.5).
    assert_eq!(ParticleObject::get_schedule_interval(100.0), 277);
    assert_eq!(ParticleObject::get_schedule_interval(1_000.0), 525);
  }
}
