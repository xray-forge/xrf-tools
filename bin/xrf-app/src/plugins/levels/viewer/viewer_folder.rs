use std::fs;
use std::path::{Path, PathBuf};

use crate::core::directories::{get_app_data_directories, get_first_unicode_path};
use crate::core::types::TauriResult;

/// A folder of the level viewer's files in the application's own data, `level-viewer/<folder>`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LevelViewerFolder {
  Views,
  Screenshots,
}

impl LevelViewerFolder {
  /// The extension of the screenshots' images.
  pub const SCREENSHOT_EXTENSION: &'static str = "png";

  /// The extension of a screenshot's report beside it.
  pub const REPORT_EXTENSION: &'static str = "json";

  fn get_name(self) -> &'static str {
    match self {
      Self::Views => "views",
      Self::Screenshots => "screenshots",
    }
  }

  /// The folder under the application's first data directory the page can address, made where it is not yet.
  ///
  /// # Errors
  ///
  /// Returns an error when no data directory has a Unicode path or the folder cannot be made.
  pub fn resolve(self, local_data: Option<PathBuf>) -> TauriResult<PathBuf> {
    let root: String = get_first_unicode_path(get_app_data_directories(local_data))
      .ok_or_else(|| String::from("No application data directory has a Unicode path"))?;
    let folder: PathBuf = PathBuf::from(root).join("level-viewer").join(self.get_name());

    fs::create_dir_all(&folder).map_err(|error| format!("Failed to make '{}': {error}", folder.display()))?;

    Ok(folder)
  }

  /// Refuses a path other than a screenshot's image directly in the screenshots folder, the only files the page may
  /// read, copy or delete through the level viewer.
  ///
  /// # Errors
  ///
  /// Returns an error naming the path when it is not one, or when the folder cannot be resolved.
  pub fn require_screenshot(local_data: Option<PathBuf>, path: &Path) -> TauriResult {
    if Self::is_screenshot(&Self::Screenshots.resolve(local_data)?, path) {
      Ok(())
    } else {
      Err(format!("'{}' is not a screenshot of the level viewer", path.display()))
    }
  }

  /// Whether `path` is an image directly in the screenshots folder (`folder`).
  fn is_screenshot(folder: &Path, path: &Path) -> bool {
    path.parent() == Some(folder)
      && path.file_name().is_some()
      && path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case(Self::SCREENSHOT_EXTENSION))
  }
}

#[cfg(test)]
mod tests {
  use std::path::Path;

  use super::LevelViewerFolder;

  #[test]
  fn takes_only_an_image_directly_in_the_screenshots_folder() {
    let folder: &Path = Path::new("appdata/level-viewer/screenshots");

    assert!(LevelViewerFolder::is_screenshot(
      folder,
      &folder.join("zaton-20261020-070509.png")
    ));
    assert!(LevelViewerFolder::is_screenshot(folder, &folder.join("zaton.PNG")));
    assert!(!LevelViewerFolder::is_screenshot(folder, &folder.join("zaton.json")));
    assert!(!LevelViewerFolder::is_screenshot(
      folder,
      &folder.join("nested").join("zaton.png")
    ));
    assert!(!LevelViewerFolder::is_screenshot(
      folder,
      &folder.join("..").join("logs.png")
    ));
    assert!(!LevelViewerFolder::is_screenshot(
      folder,
      Path::new("C:/Windows/zaton.png")
    ));
  }
}
