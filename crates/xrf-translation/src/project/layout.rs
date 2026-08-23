use std::fs;
use std::path::{Path, PathBuf};

use crate::language::TranslationLanguage;
use crate::project::constants::MAP_DESC_DIRECTORY;
use crate::project::descriptor::TranslationProjectMode;
use crate::{json, xml};
use xrf_utils::to_portable_path_string;

/// Report which layout a directory looks like.
///
/// Advisory only. The mode a project is opened with is whatever the caller passes, because the two
/// layouts save to different files and a heuristic must not be what decides that.
pub fn detect_mode(root: &Path) -> TranslationProjectMode {
  let Ok(entries) = fs::read_dir(root) else {
    return TranslationProjectMode::Source;
  };

  let mut has_language_directory: bool = false;

  for entry in entries.filter_map(Result::ok) {
    let path: PathBuf = entry.path();
    let name: String = entry.file_name().to_string_lossy().into_owned();

    if path.is_dir() && name != MAP_DESC_DIRECTORY && TranslationLanguage::from_str_single(&name).is_ok() {
      has_language_directory = true;
    }

    // A source tree is recognised by what only it has: JSON maps and language-suffixed XML.
    if path.is_file() {
      match path.extension().and_then(|extension| extension.to_str()) {
        Some(json::FILE_EXTENSION) => return TranslationProjectMode::Source,
        Some(xml::FILE_EXTENSION) if TranslationLanguage::from_file_name(&path).is_some() => {
          return TranslationProjectMode::Source;
        }
        _ => {}
      }
    }
  }

  if has_language_directory {
    TranslationProjectMode::Gamedata
  } else {
    TranslationProjectMode::Source
  }
}

pub(crate) fn relative(root: &Path, path: &Path) -> String {
  to_portable_path_string(path.strip_prefix(root).unwrap_or(path))
}
