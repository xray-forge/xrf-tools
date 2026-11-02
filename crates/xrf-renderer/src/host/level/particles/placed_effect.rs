/// An effect a placed particle system plays: a planted system or a zone plays only its idle one, and a campfire its
/// disabled and enabling ones besides.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlacedEffect {
  /// The planted effect, or a zone's `idle_particles`.
  Idle,
  /// A campfire's `disabled_particles`, played once as it goes out.
  Disabled,
  /// A campfire's `enabling_particles`, played as it is lit until its idle effect takes over.
  Enabling,
}

impl PlacedEffect {
  /// Every one, in slot order.
  pub const ALL: [PlacedEffect; 3] = [PlacedEffect::Idle, PlacedEffect::Disabled, PlacedEffect::Enabling];

  /// How many there are, which an emitter keeps a slot each for.
  pub const COUNT: usize = Self::ALL.len();

  /// Its slot among a system's, in declaration order.
  pub fn get_index(self) -> usize {
    self as usize
  }
}
