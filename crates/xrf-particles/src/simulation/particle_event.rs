use crate::simulation::particle::Particle;

/// A birth or death the engine reports through `b_cb` and `d_cb`, in the order it happened.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ParticleEvent {
  /// A particle appended at the end of the effect, as it was born.
  Birth(Particle),
  /// The particle at an index, as it died, before the last one was moved into its place.
  Death { index: usize, particle: Particle },
}
