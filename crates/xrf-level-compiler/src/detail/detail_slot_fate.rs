use xrf_level::LevelDetailsLight;

/// What the detail lighter does with one slot (`LightThread::Execute`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DetailSlotFate {
  /// It plants nothing, and is left as it is.
  Unplanted,
  /// It plants something at zero density in every corner: its corners are cleared and it is left unlit.
  Emptied,
  /// Its box touches no face of the ray model, and it is left unlit.
  Untouched,
  /// Lit.
  Lit(LevelDetailsLight),
}
