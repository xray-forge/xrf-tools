use serde::Deserialize;

/// How a level map's annotations, the SVG beside its capture, go into an export of it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum MapExportAnnotations {
  /// Drawn over the map.
  Over,
  /// Alone, clear where they draw nothing.
  Alone,
}
