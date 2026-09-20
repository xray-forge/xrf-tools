use serde::Serialize;
use xrf_vfs::XrayResolution;

/// One motion file set a visual animates from, and what the reference came to.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualMotionDependency {
  pub reference: String,
  pub resolution: XrayResolution,
}
