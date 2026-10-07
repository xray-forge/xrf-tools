use serde::{Deserialize, Serialize};

/// Where a camera is and the point it looks at.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct WorldCameraPose {
  pub position: [f32; 3],
  pub target: [f32; 3],
}
