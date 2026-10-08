use serde::{Deserialize, Serialize};

/// How many times the sky debanding averages a pixel with its neighbours, each time further out.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderDebandingQuality {
  /// Once.
  Low,
  /// Twice.
  #[default]
  Medium,
  /// Three times.
  High,
  /// Four times.
  Ultra,
}

impl RenderDebandingQuality {
  /// Every quality, cheapest first.
  pub const ALL: [Self; 4] = [Self::Low, Self::Medium, Self::High, Self::Ultra];

  /// Times a pixel is averaged with its neighbours.
  pub const fn get_passes(self) -> u32 {
    match self {
      Self::Low => 1,
      Self::Medium => 2,
      Self::High => 3,
      Self::Ultra => 4,
    }
  }
}
