use serde::Serialize;
use xrf_app_core::TauriResult;
use xrf_vfs::{XrayDirectoryListing, XrayLogicalPath, XraySearch};

use crate::level::{LevelBundle, LevelSource};
use crate::parts::LevelGeometryPart;

/// One compiled level the roots hold, as a picker lists it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelEntry {
  /// The name the installation knows the level by, which is its directory under `levels`.
  pub name: String,
  /// Its engine identity, which is what opening it takes.
  pub logical_path: String,
  /// Whether `level.geom` sits beside the bundle; a level without it draws nothing.
  pub has_geometry: bool,
}

impl LevelEntry {
  /// Every compiled level `search` reaches: each directory under `levels` holding a bundle, loose or archived alike.
  ///
  /// # Errors
  ///
  /// Returns an error when `levels` cannot be listed.
  pub fn list(search: &XraySearch) -> TauriResult<Vec<Self>> {
    let levels: XrayLogicalPath =
      XrayLogicalPath::new(LevelSource::LEVELS_DIRECTORY).map_err(|error| format!("Failed to list levels: {error}"))?;
    let listing: XrayDirectoryListing = search
      .list_children(levels.as_str())
      .map_err(|error| format!("Failed to list levels: {error}"))?;

    Ok(
      listing
        .directories
        .into_iter()
        .filter_map(|name| {
          let logical_path: XrayLogicalPath = levels.join(&name).ok()?;

          Self::is_present(search, &logical_path, LevelBundle::FILE).then(|| Self {
            // A level shipping no render geometry draws nothing, which is worth saying in the list rather than on open.
            has_geometry: Self::is_present(search, &logical_path, LevelGeometryPart::FILE),
            logical_path: logical_path.as_str().to_owned(),
            name,
          })
        })
        .collect(),
    )
  }

  /// Whether one of a level's files is there, treating an unreadable path as absent rather than failing the listing.
  fn is_present(search: &XraySearch, logical_path: &XrayLogicalPath, file: &str) -> bool {
    logical_path
      .join(file)
      .and_then(|path| search.find(path.as_str()))
      .is_ok_and(|found| found.is_some())
  }
}
