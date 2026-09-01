//! The boundary bulk extraction writes through: one destination root that only ever grows downwards.

use std::ffi::OsStr;
use std::fs;
use std::fs::{File, Metadata};
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

/// A destination root an archive-controlled name cannot write outside of.
///
/// Checking the name is not enough. [`crate::path::to_host_relative`] proves an entry spells no traversal, but it says
/// nothing about the tree the name lands in: `create_dir_all` walks an existing symlink and `File::open` follows one,
/// so `configs\system.ltx` still writes wherever a pre-existing `out\configs` link points. Containment is therefore a
/// property of the writer, not of the name.
///
/// Every component below the root is created or verified one at a time, and an existing link is refused rather than
/// descended through. Nothing here resolves a path and compares it against the root: a resolved path is stale the
/// moment it is returned, so a component swapped after the check would still be walked.
///
/// The root itself is never inspected. The caller named it, and a caller extracting into a linked directory means it.
pub(crate) struct RootedDestination {
  root: PathBuf,
}

impl RootedDestination {
  pub(crate) fn new(root: &Path) -> Self {
    Self { root: root.into() }
  }

  /// Creates the root itself, once, before anything is written below it.
  ///
  /// Separate from [`Self::create_directory`] because that one runs per entry, and creating a directory that already
  /// exists is still a syscall. The root is not inspected for links, for the reason the type's own documentation gives.
  ///
  /// # Errors
  ///
  /// Returns an IO error when the root cannot be created.
  pub(crate) fn create_root(&self) -> XrfResult {
    fs::create_dir_all(&self.root)?;

    Ok(())
  }

  pub(crate) fn get_root(&self) -> &Path {
    &self.root
  }

  /// Creates the directory `relative` names below the root and returns its host path.
  ///
  /// An empty `relative` names the root itself, which is what a volume's entry-point directory record spells.
  ///
  /// # Errors
  ///
  /// Returns an invalid error naming the component that is a link or is not a directory.
  pub(crate) fn create_directory(&self, relative: &Path) -> XrfResult<PathBuf> {
    let mut path: PathBuf = self.root.clone();

    for component in relative.components() {
      path.push(Self::descend(component, relative)?);

      Self::create_component(&path)?;
    }

    Ok(path)
  }

  /// Opens the file `relative` names below the root for writing, creating the directories above it.
  ///
  /// # Errors
  ///
  /// Returns an invalid error when any component on the way down, or the file itself, is an existing link.
  pub(crate) fn create_file(&self, relative: &Path) -> XrfResult<File> {
    let Some(name) = relative.file_name() else {
      return Err(XrfError::new_invalid_error(format!(
        "Archive entry path '{}' names no file to write",
        format_path(relative)
      )));
    };

    self.create_file_in(
      &self.create_directory(relative.parent().unwrap_or(Path::new("")))?,
      name,
    )
  }

  /// Opens `name` inside a parent directory this run already walked down to.
  ///
  /// # Errors
  ///
  /// Returns an invalid error when `parent` is not below the root, or when a link already occupies the file's name.
  pub(crate) fn create_file_in(&self, parent: &Path, name: &OsStr) -> XrfResult<File> {
    // Lexical, and no syscall: the guarantee this type exists for is that nothing writes outside the root, so a parent
    // reaching it from anywhere else is refused rather than trusted.
    if !parent.starts_with(&self.root) {
      return Err(XrfError::new_invalid_error(format!(
        "Refusing to write into '{}': it is not below the destination root '{}'",
        format_path(parent),
        format_path(&self.root)
      )));
    }

    let path: PathBuf = parent.join(name);

    // Overwriting a file already there is ordinary, following a link that took its place is not.
    match fs::symlink_metadata(&path) {
      Ok(metadata) if is_link(&metadata) => return Err(Self::new_link_error(&path)),
      Ok(_) => {}
      Err(error) if error.kind() == ErrorKind::NotFound => {}
      Err(error) => return Err(error.into()),
    }

    Ok(
      File::options()
        .read(false)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?,
    )
  }

  /// The one step down, refused unless it is an ordinary name.
  ///
  /// `to_host_relative` already rejects `.`, `..`, and a drive before a name becomes a path, but containment is this
  /// type's to keep: a second caller reaching this without that helper must not be trusted to have spelled it safely.
  fn descend<'a>(component: Component<'a>, relative: &Path) -> XrfResult<&'a Path> {
    match component {
      Component::Normal(name) => Ok(Path::new(name)),
      _ => Err(XrfError::new_invalid_error(format!(
        "Archive entry path '{}' contains a component that would leave its destination",
        format_path(relative)
      ))),
    }
  }

  /// Makes one component a real directory without walking through whatever already occupies it.
  fn create_component(path: &Path) -> XrfResult {
    match fs::symlink_metadata(path) {
      Ok(metadata) => Self::ensure_directory(path, &metadata),
      Err(error) if error.kind() == ErrorKind::NotFound => match fs::create_dir(path) {
        Ok(()) => Ok(()),
        // Another worker of the same run got there first; what it left still has to be a real directory.
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {
          Self::ensure_directory(path, &fs::symlink_metadata(path)?)
        }
        Err(error) => Err(error.into()),
      },
      Err(error) => Err(error.into()),
    }
  }

  fn ensure_directory(path: &Path, metadata: &Metadata) -> XrfResult {
    if is_link(metadata) {
      return Err(Self::new_link_error(path));
    }

    if metadata.is_dir() {
      Ok(())
    } else {
      Err(XrfError::new_invalid_error(format!(
        "Cannot extract into '{}' - it already exists and is not a directory",
        format_path(path)
      )))
    }
  }

  fn new_link_error(path: &Path) -> XrfError {
    XrfError::new_invalid_error(format!(
      "Cannot extract through '{}' - it is a link, and extraction stays inside the destination it was given",
      format_path(path)
    ))
  }
}

/// Whether an existing entry redirects a write somewhere the path does not spell.
///
/// Windows reads more than a symlink that way: a junction redirects the same traversal, and an unrecognized reparse
/// tag is handled by a filter driver this crate knows nothing about. The attribute covers all of them, where
/// `is_symlink` covers only the symlink and mount-point tags.
#[cfg(windows)]
fn is_link(metadata: &Metadata) -> bool {
  use std::os::windows::fs::MetadataExt;

  const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;

  metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn is_link(metadata: &Metadata) -> bool {
  metadata.is_symlink()
}
