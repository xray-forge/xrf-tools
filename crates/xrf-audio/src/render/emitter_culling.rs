/// What a loudness update answers: whether the emitter is loud enough for a voice, and how much it deserves one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct EmitterCulling {
  pub(crate) is_audible: bool,
  pub(crate) priority: f32,
}
