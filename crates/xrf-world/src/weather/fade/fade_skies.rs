/// Whether the skies a fade fades into are up yet, which it waits for before it starts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FadeSkies {
  Up,
  Waited,
}

impl FadeSkies {
  /// Up while `is_up`, else waited for.
  pub fn of(is_up: bool) -> Self {
    if is_up { Self::Up } else { Self::Waited }
  }
}
