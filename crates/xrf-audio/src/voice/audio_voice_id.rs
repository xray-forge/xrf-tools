/// A playing voice: its slot in the mixer's pool and which use of that slot it is, so a command for a voice that has
/// ended never reaches the one playing in its slot after it.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AudioVoiceId {
  pub(crate) slot: u32,
  pub(crate) generation: u32,
}
