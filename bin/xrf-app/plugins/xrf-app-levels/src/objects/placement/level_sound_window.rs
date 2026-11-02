use serde::Serialize;
use xrf_level::LevelSndStaticWindow;

/// One of a static sound's windows, in milliseconds, as `SStaticSound` compares a clock against it: from and to, both
/// nought for the sound's own default (all day, its whole length, looped without a pause).
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSoundWindow {
  pub from: u32,
  pub to: u32,
}

impl From<&LevelSndStaticWindow> for LevelSoundWindow {
  fn from(window: &LevelSndStaticWindow) -> Self {
    Self {
      from: window.from,
      to: window.to,
    }
  }
}
