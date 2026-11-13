use serde::{Deserialize, Serialize};

/// The GPU the detail lighter's GPU backend traces on, by its PCI vendor and device ids, as the renderer's settings name
/// one (`RenderGpu`).
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailGpu {
  pub vendor: u32,
  pub device: u32,
}
