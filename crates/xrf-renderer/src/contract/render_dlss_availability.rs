use serde::{Deserialize, Serialize};

/// Whether a viewport's GPU upscales with DLSS, or reconstructs rays with it, and where it does not, what it lacks.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderDlssAvailability {
  Available,
  /// The build has no DLSS: it was built without the `dlss` feature.
  #[default]
  Unbuilt,
  /// The GPU or its driver has no DLSS, or none of the feature.
  Unsupported,
}
