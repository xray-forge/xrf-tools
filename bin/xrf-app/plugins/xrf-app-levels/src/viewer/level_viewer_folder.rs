use std::fs;
use std::path::{Path, PathBuf};

use xrf_app_core::TauriResult;
use xrf_app_core::directories::AppDirectories;
use xrf_utils::HostFileName;

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

  /// The extension of a level view's file.
  pub const VIEW_EXTENSION: &'static str = "json";

  /// The extension of a level map's annotations beside it.
  pub const ANNOTATIONS_EXTENSION: &'static str = "svg";

  fn get_name(self) -> &'static str {
    match self {
      Self::Views => "views",
      Self::Screenshots => "screenshots",
    }
  }

  /// The folder under the application's first data directory the page can address, whether or not it is there yet.
  ///
  /// # Errors
  ///
  /// Returns an error when no data directory has a Unicode path.
  pub fn get_path(self, directories: &AppDirectories) -> TauriResult<PathBuf> {
    let root: String = directories
      .get_app_data_root()
      .ok_or_else(|| String::from("No application data directory has a Unicode path"))?;

    Ok(PathBuf::from(root).join("level-viewer").join(self.get_name()))
  }

  /// `folder`, as `get_path` names it, made where it is not yet, for a command handing it to the page; it touches the
  /// disk, so it runs on the pool.
  ///
  /// # Errors
  ///
  /// Returns an error when the folder cannot be made.
  pub fn make(folder: PathBuf) -> TauriResult<PathBuf> {
    fs::create_dir_all(&folder).map_err(|error| format!("Failed to make '{}': {error}", folder.display()))?;

    Ok(folder)
  }

  /// Refuses a path other than a screenshot's image directly in the screenshots folder, the only files the page may
  /// have written, read, copied or deleted in that folder through the level viewer; checks the path alone, making
  /// nothing.
  ///
  /// # Errors
  ///
  /// Returns an error naming the path when it is not one, or when the folder cannot be named.
  pub fn require_screenshot(directories: &AppDirectories, path: &Path) -> TauriResult {
    if Self::is_screenshot(&Self::Screenshots.get_path(directories)?, path) {
      Ok(())
    } else {
      Err(format!("'{}' is not a screenshot of the level viewer", path.display()))
    }
  }

  /// Refuses a view file other than a JSON one, the only kind the page may write or read where a file dialog answered.
  ///
  /// # Errors
  ///
  /// Returns an error naming the path when it is not a JSON file.
  pub fn require_view_file(path: &Path) -> TauriResult {
    if path.file_stem().is_some()
      && path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case(Self::VIEW_EXTENSION))
    {
      Ok(())
    } else {
      Err(format!("'{}' is not a level view's JSON file", path.display()))
    }
  }

  /// The first free path in the screenshots folder (`folder`) for a screenshot named `name`, `-2`, `-3` and on added
  /// for one taken within the same second.
  ///
  /// # Errors
  ///
  /// Returns an error when `name` is not one file name Windows takes, or starts with a dot.
  pub fn find_free_screenshot(folder: &Path, name: &str) -> TauriResult<PathBuf> {
    let name: HostFileName = HostFileName::new(name, "A screenshot's name").map_err(|error| error.to_string())?;

    if name.as_str().contains(['*', '?', '"', '<', '>', '|']) || name.as_str().starts_with('.') {
      return Err(format!("'{name}' is not a screenshot's name"));
    }

    (1..=u32::MAX)
      .map(|index: u32| {
        let stem: String = if index == 1 {
          name.to_string()
        } else {
          format!("{name}-{index}")
        };

        folder.join(format!("{stem}.{}", Self::SCREENSHOT_EXTENSION))
      })
      .find(|path: &PathBuf| !path.exists())
      .ok_or_else(|| format!("No free name for the screenshot '{name}'"))
  }

  /// Where the report of the screenshot at `path` stands beside it.
  pub fn get_report_path(path: &Path) -> PathBuf {
    path.with_extension(Self::REPORT_EXTENSION)
  }

  /// Where a level map's annotations, captured at `path`, stand beside it.
  pub fn get_annotations_path(path: &Path) -> PathBuf {
    path.with_extension(Self::ANNOTATIONS_EXTENSION)
  }

  /// Whether `path` is an image directly in the screenshots folder (`folder`).
  pub(crate) fn is_screenshot(folder: &Path, path: &Path) -> bool {
    path.parent() == Some(folder)
      && path.file_name().is_some()
      && path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case(Self::SCREENSHOT_EXTENSION))
  }
}
