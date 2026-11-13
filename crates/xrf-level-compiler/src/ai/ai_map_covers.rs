use serde::{Deserialize, Serialize};

/// Which covers the AI map compiler computes.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AiMapCovers {
  /// None, as xrAI's `-draft` leaves them: every value 0, and no `build.cform` needed.
  None,
  /// Each corner node traced against `build.cform` and the other nodes mixed from them, as xrAI's full run does.
  #[default]
  Full,
}

impl AiMapCovers {
  /// Every choice, in the order a command line lists them.
  pub const ALL: [Self; 2] = [Self::None, Self::Full];

  /// The choice's name on a command line, `none` or `full`.
  pub fn get_name(self) -> &'static str {
    match self {
      Self::None => "none",
      Self::Full => "full",
    }
  }

  /// The choice a command line names, by [`Self::get_name`]; none for another name.
  pub fn from_name(name: &str) -> Option<Self> {
    Self::ALL.into_iter().find(|covers| covers.get_name() == name)
  }
}
