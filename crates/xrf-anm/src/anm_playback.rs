/// How an object animator plays a motion, `CObjectAnimator::Play`'s `bLoop`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnmPlayback {
  /// Plays the motion once and holds its last frame's time, as a camera effect that is not cyclic does.
  Once,
  /// Wraps back by whole lengths once past the last frame's time, as a torrid zone's motion does.
  Looped,
}
