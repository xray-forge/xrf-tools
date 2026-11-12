use xrf_level::LevelDetailsLight;

/// What the detail lighter does with one slot (`LightThread::Execute`, `xrDO_Light/LightThread.cpp:288`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DetailSlotFate {
  /// It plants nothing, and is left as it is.
  Unplanted,
  /// It plants something, but every planting corner's densities are zero: its corners are cleared and it is left
  /// unlit.
  Emptied,
  /// Its box touches no face of the ray model, and it is left unlit.
  Untouched,
  /// Lit.
  Lit(LevelDetailsLight),
}
