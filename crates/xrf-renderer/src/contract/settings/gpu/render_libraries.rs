use serde::{Deserialize, Serialize};

/// Graphics libraries shipped beside the application; currently only the DLSS runtime.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderLibraries {
  /// The DLSS runtime's file version, for example `310.9.1.0`; none if the build has no DLSS or the library is missing
  /// from the bundle.
  pub dlss: Option<String>,
}
