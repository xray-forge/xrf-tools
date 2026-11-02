/// How the engine plays an animation: once, or over and over, as its animators' `bLoop` and `m_bCyclic` choose. What
/// either means for a format's time is that format's own rule, such as `AnmFile::get_played_time`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationPlayback {
  /// Played once, as a camera or post-process effect that is not cyclic is.
  Once,
  /// Played over and over, as a torrid zone's motion and a cyclic effect are.
  Looped,
}
