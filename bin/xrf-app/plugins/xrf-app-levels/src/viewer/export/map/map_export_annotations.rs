use serde::Deserialize;

/// How a map export includes the annotations SVG written beside its capture.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum MapExportAnnotations {
  /// Drawn over the map.
  Over,
  /// Alone, transparent where they draw nothing.
  Alone,
}
