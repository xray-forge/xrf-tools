/// How an effect stops: at once with its particles (`Stop(FALSE)`, then `Destroy`), or deferred, emitting no more while
/// what it emitted lives out its life (`Stop(TRUE)`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParticleStop {
  AtOnce,
  Deferred,
}

impl ParticleStop {
  /// Deferred where `is_deferred`, else at once.
  pub fn of(is_deferred: bool) -> Self {
    if is_deferred { Self::Deferred } else { Self::AtOnce }
  }
}
