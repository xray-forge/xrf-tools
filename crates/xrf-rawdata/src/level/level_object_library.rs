use std::collections::BTreeMap;
use std::io::{Error as IoError, ErrorKind};
use std::path::{Path, PathBuf};

use byteorder::ByteOrder;
use walkdir::{DirEntry, WalkDir};
use xrf_editable_object::EditableObjectFile;
use xrf_error::{XrfError, XrfResult};
use xrf_job::JobHandle;
use xrf_utils::format_path;

use crate::level::level_object_library_entry::LevelObjectLibraryEntry;
use crate::provenance::level_scene_refusal::LevelSceneRefusal;

/// The `.object` files under a decompile's object library roots, listed once for every step, and each one that could
/// not be read, with why.
///
/// Each root is walked in name order, so results match on every machine, following links and junctions; a link back
/// to a folder above it, or to nothing, is recorded, not followed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LevelObjectLibrary {
  entries: Vec<LevelObjectLibraryEntry>,
  /// What was not read, by file: a file failing for two steps is recorded once.
  unread: BTreeMap<PathBuf, String>,
}

impl LevelObjectLibrary {
  const OBJECT_EXTENSION: &'static str = "object";

  /// The objects under each of `roots`, in their order, each root's in name order.
  ///
  /// # Errors
  ///
  /// Returns an error when a root is not a directory, or a folder under one cannot be listed.
  pub fn list(roots: &[PathBuf]) -> XrfResult<Self> {
    let mut library: Self = Self::default();

    for root in roots {
      if !root.is_dir() {
        return Err(XrfError::new_not_found_error(format!(
          "Object library {} is not a directory",
          format_path(root)
        )));
      }

      library.list_root(root)?;
    }

    Ok(library)
  }

  /// Every object listed, in order.
  #[cfg(test)]
  pub(crate) fn get_entries(&self) -> &[LevelObjectLibraryEntry] {
    &self.entries
  }

  /// Reads each object `is_wanted` keeps once, in order, and hands it to `visit`; one that does not read is recorded
  /// and skipped. A large library takes a while, so `job` is checked before each object.
  ///
  /// # Errors
  ///
  /// Returns the first error `visit` returns, or a cancelled error once `job` is cancelled.
  pub fn read_each<T: ByteOrder>(
    &mut self,
    job: &JobHandle,
    is_wanted: impl Fn(&LevelObjectLibraryEntry) -> bool,
    mut visit: impl FnMut(&LevelObjectLibraryEntry, EditableObjectFile) -> XrfResult,
  ) -> XrfResult {
    let Self { entries, unread } = self;

    for entry in entries.iter().filter(|entry| is_wanted(entry)) {
      job.check_cancelled()?;

      match EditableObjectFile::read_from_path::<T, _>(&entry.path) {
        Ok(object) => visit(entry, object)?,
        Err(error) => Self::record(unread, &entry.path, error.to_string()),
      }
    }

    Ok(())
  }

  /// Records that the file at `path` was not used for `cause`, unless it already was.
  pub(crate) fn record_unread(&mut self, path: &Path, cause: String) {
    Self::record(&mut self.unread, path, cause);
  }

  /// What was not read, as the decompile's refusals: each file, and why nothing in the scene can come from it.
  pub fn get_refusals(&self) -> Vec<LevelSceneRefusal> {
    self
      .unread
      .iter()
      .map(|(path, cause)| LevelSceneRefusal {
        file: format_path(path).to_string(),
        object: path
          .file_name()
          .map(|name| name.to_string_lossy().into_owned())
          .unwrap_or_default(),
        reason: format!("The object library's file was not used, so nothing in the scene matches it: {cause}"),
      })
      .collect()
  }

  fn record(unread: &mut BTreeMap<PathBuf, String>, path: &Path, cause: String) {
    unread.entry(path.to_path_buf()).or_insert(cause);
  }

  fn list_root(&mut self, root: &Path) -> XrfResult {
    for walked in WalkDir::new(root).follow_links(true).sort_by_file_name() {
      let entry: DirEntry = match walked {
        Ok(entry) => entry,
        Err(error) => {
          // A link is the user's layout: one looping back or left dangling is a file not used, where a folder that
          // cannot be listed leaves the library unknown.
          let cause: Option<String> = match (error.loop_ancestor(), error.io_error().map(IoError::kind)) {
            (Some(ancestor), _) => Some(format!("it links back to {}, a folder above it", format_path(ancestor))),
            (None, Some(ErrorKind::NotFound)) => Some(String::from("it links to nothing that exists")),
            _ => None,
          };

          match (cause, error.path()) {
            (Some(cause), Some(path)) => {
              self.record_unread(path, cause);
              continue;
            }
            _ => {
              return Err(XrfError::new_read_error(format!(
                "Object library {} was not listed: {error}",
                format_path(root)
              )));
            }
          }
        }
      };

      if entry.file_type().is_file()
        && entry
          .path()
          .extension()
          .is_some_and(|extension| extension.eq_ignore_ascii_case(Self::OBJECT_EXTENSION))
      {
        self.entries.push(LevelObjectLibraryEntry::new(root, entry.into_path()));
      }
    }

    Ok(())
  }
}
