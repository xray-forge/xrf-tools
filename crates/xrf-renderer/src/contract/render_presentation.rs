use serde::{Deserialize, Serialize};

/// How often frames are presented.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderPresentation {
  /// One frame a refresh of the display.
  #[default]
  Vsync,
  /// As fast as the frame can be drawn, for measuring.
  Uncapped,
}
