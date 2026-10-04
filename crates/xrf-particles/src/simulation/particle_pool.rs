use glam::{Vec3, Vec4};

use crate::simulation::particle::Particle;
use crate::simulation::particle_event::ParticleEvent;
use crate::simulation::particle_random::ParticleRandom;

/// `PAPI::ParticleEffect`: an effect's live particles, its random sequence and its birth rules.
#[derive(Clone, Debug)]
pub struct ParticlePool {
  particles: Vec<Particle>,
  max_particles: usize,
  random: ParticleRandom,
  /// `m_iFrameCount` when `dfRandomFrame` starts each particle on a random frame.
  random_frames: Option<i32>,
  /// `dfAnimated` with `dfRandomPlayback`: half the particles play their frames backwards.
  is_random_playback: bool,
  /// Births and deaths since last taken, kept only while the owner listens (`b_cb`, `d_cb`).
  events: Option<Vec<ParticleEvent>>,
}

impl ParticlePool {
  pub(crate) fn new(max_particles: usize, seed: i32) -> Self {
    Self {
      particles: Vec::with_capacity(max_particles),
      max_particles,
      random: ParticleRandom::new(seed),
      random_frames: None,
      is_random_playback: false,
      events: None,
    }
  }

  /// Starts each particle born on a random one of a number of frames.
  pub(crate) fn with_random_frames(mut self, frame_count: i32) -> Self {
    self.random_frames = Some(frame_count);
    self
  }

  /// Plays half the particles born backwards.
  pub(crate) fn with_random_playback(mut self) -> Self {
    self.is_random_playback = true;
    self
  }

  /// Keeps births and deaths for the owner to take.
  pub(crate) fn with_events(mut self) -> Self {
    self.events = Some(Vec::new());
    self
  }

  pub fn len(&self) -> usize {
    self.particles.len()
  }

  pub fn is_empty(&self) -> bool {
    self.particles.is_empty()
  }

  pub fn get_max_particles(&self) -> usize {
    self.max_particles
  }

  pub fn get_particles(&self) -> &[Particle] {
    &self.particles
  }

  pub(crate) fn get_particles_mut(&mut self) -> &mut [Particle] {
    &mut self.particles
  }

  pub(crate) fn get_random(&mut self) -> &mut ParticleRandom {
    &mut self.random
  }

  /// The particles and the random sequence at once, for an action drawing a number per particle.
  pub(crate) fn split_mut(&mut self) -> (&mut [Particle], &mut ParticleRandom) {
    (&mut self.particles, &mut self.random)
  }

  /// `Add`: appends a particle unless the effect is full, then applies the birth rules and reports it.
  #[allow(clippy::too_many_arguments)]
  pub(crate) fn add(
    &mut self,
    position: Vec3,
    previous_position: Vec3,
    size: Vec3,
    rotation: Vec3,
    velocity: Vec3,
    color: Vec4,
    age: f32,
  ) -> bool {
    if self.particles.len() >= self.max_particles {
      return false;
    }

    let mut particle: Particle = Particle {
      rotation: rotation.x,
      position,
      previous_position,
      velocity,
      size,
      color,
      age,
      frame: 0,
      is_reversed: false,
    };

    if let Some(frame_count) = self.random_frames {
      particle.frame = (self.random.next_below(frame_count) as f32 * 255.0).floor() as u16;
    }

    if self.is_random_playback && self.random.next_below(2) != 0 {
      particle.is_reversed = true;
    }

    if let Some(events) = &mut self.events {
      events.push(ParticleEvent::Birth(particle));
    }

    self.particles.push(particle);

    true
  }

  /// `Remove`: reports the particle's death, then moves the last one into its place.
  pub(crate) fn remove(&mut self, index: usize) {
    if self.particles.is_empty() {
      return;
    }

    if let Some(events) = &mut self.events {
      events.push(ParticleEvent::Death {
        index,
        particle: self.particles[index],
      });
    }

    self.particles.swap_remove(index);
  }

  /// `p_count = 0`: drops every particle at once, reporting none, as a stop without delay does.
  pub(crate) fn clear(&mut self) {
    self.particles.clear();
  }

  /// The births and deaths since last taken, oldest first.
  pub(crate) fn take_events(&mut self) -> Vec<ParticleEvent> {
    self.events.as_mut().map(std::mem::take).unwrap_or_default()
  }
}

#[cfg(test)]
mod tests {
  use glam::{Vec3, Vec4};

  use super::ParticlePool;
  use crate::simulation::particle_event::ParticleEvent;

  fn add_at(pool: &mut ParticlePool, x: f32) -> bool {
    pool.add(
      Vec3::new(x, 0.0, 0.0),
      Vec3::ZERO,
      Vec3::ONE,
      Vec3::ZERO,
      Vec3::ZERO,
      Vec4::ONE,
      0.0,
    )
  }

  #[test]
  fn refuses_a_particle_past_the_limit() {
    let mut pool: ParticlePool = ParticlePool::new(2, 1);

    assert!(add_at(&mut pool, 0.0));
    assert!(add_at(&mut pool, 1.0));
    assert!(!add_at(&mut pool, 2.0));
    assert_eq!(pool.len(), 2);
  }

  #[test]
  fn moves_the_last_particle_into_a_removed_one() {
    let mut pool: ParticlePool = ParticlePool::new(3, 1).with_events();

    add_at(&mut pool, 0.0);
    add_at(&mut pool, 1.0);
    add_at(&mut pool, 2.0);
    pool.remove(0);

    assert_eq!(pool.get_particles()[0].position.x, 2.0);
    assert_eq!(pool.get_particles()[1].position.x, 1.0);

    let events: Vec<ParticleEvent> = pool.take_events();

    assert_eq!(events.len(), 4);
    assert!(matches!(events[3], ParticleEvent::Death { index: 0, particle } if particle.position.x == 0.0));
    assert!(pool.take_events().is_empty());
  }

  #[test]
  fn starts_a_particle_on_a_random_frame_times_255() {
    // `randI(16)` from seed one is 41 % 16 = 9.
    let mut pool: ParticlePool = ParticlePool::new(1, 1).with_random_frames(16);

    add_at(&mut pool, 0.0);

    assert_eq!(pool.get_particles()[0].frame, 9 * 255);
  }
}
