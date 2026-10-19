use std::ffi::OsStr;
use std::fs;
use std::fs::{File, Metadata};
use std::io::{ErrorKind, Result as IoResult};
use std::path::{Component, Path, PathBuf};

use xrf_error::{XrfError, XrfResult};

use crate::host_path::{HostLinkPolicy, HostRelativePath};
use crate::path_utils::format_path;

/// A destination root that data-named writes only ever grow downwards from.
pub struct HostDestination {
  root: PathBuf,
  link_policy: HostLinkPolicy,
}

impl HostDestination {
  pub fn new(root: &Path, link_policy: HostLinkPolicy) -> Self {
    Self {
      root: root.into(),
      link_policy,
    }
  }

  /// Creates the root itself, once, before anything is written below it.
  ///
  /// # Errors
  ///
  /// Returns an IO error when the root cannot be created.
  pub fn create_root(&self) -> XrfResult {
    fs::create_dir_all(&self.root)?;

    Ok(())
  }

  pub fn get_root(&self) -> &Path {
    &self.root
  }

  pub fn get_link_policy(&self) -> HostLinkPolicy {
    self.link_policy
  }

  /// Creates the directory `relative` names below the root and returns its host path.
  ///
  /// # Errors
  ///
  /// Returns an invalid error naming a component that is not a directory, or a link the policy refuses.
  pub fn create_directory(&self, relative: &HostRelativePath) -> XrfResult<PathBuf> {
    let mut path: PathBuf = self.root.clone();

    for component in relative.as_path().components() {
      path.push(Self::descend(component, relative.as_path())?);

      self.create_component(&path)?;
    }

    Ok(path)
  }

  /// Opens the file `relative` names below the root for writing, creating the directories above it.
  ///
  /// # Errors
  ///
  /// Returns an invalid error when `relative` names no file, or a component on the way down is refused.
  pub fn create_file(&self, relative: &HostRelativePath) -> XrfResult<File> {
    let (parent, name) = self.create_parent(relative)?;

    self.create_file_in(&parent, name)
  }

  /// Creates the directories above `relative` and returns its host path, for a writer that opens files by path.
  ///
  /// # Errors
  ///
  /// Returns an invalid error when `relative` names no file, or a component on the way down is refused.
  pub fn prepare_file(&self, relative: &HostRelativePath) -> XrfResult<PathBuf> {
    let (parent, name) = self.create_parent(relative)?;
    let path: PathBuf = parent.join(name);

    self.check_file_slot(&path)?;

    Ok(path)
  }

  /// The host path of the file `relative` names, checked as far as it exists and creating nothing.
  ///
  /// # Errors
  ///
  /// Returns an invalid error when `relative` names no file, or an existing component on the way down is refused.
  pub fn resolve_file(&self, relative: &HostRelativePath) -> XrfResult<PathBuf> {
    if relative.file_name().is_none() {
      return Err(XrfError::new_invalid_error(format!(
        "Destination path '{relative}' names no file to write"
      )));
    }

    let mut path: PathBuf = self.root.clone();

    for component in relative.parent().unwrap_or_default().as_path().components() {
      path.push(Self::descend(component, relative.as_path())?);

      match self.read_metadata(&path) {
        Ok(metadata) => self.ensure_directory(&path, &metadata)?,
        // Nothing below a missing directory exists, so there is nothing left to check.
        Err(error) if error.kind() == ErrorKind::NotFound => break,
        Err(error) => return Err(error.into()),
      }
    }

    let path: PathBuf = self.root.join(relative);

    self.check_file_slot(&path)?;

    Ok(path)
  }

  /// Opens `name` inside a parent directory this run already walked down to.
  ///
  /// # Errors
  ///
  /// Returns an invalid error when `parent` is not below the root, `name` is not one file name, or a link the policy
  /// refuses already occupies it.
  pub fn create_file_in(&self, parent: &Path, name: &OsStr) -> XrfResult<File> {
    // Lexical and free: a parent reaching the root from anywhere else is refused rather than trusted.
    if !parent.starts_with(&self.root) {
      return Err(XrfError::new_invalid_error(format!(
        "Refusing to write into '{}': it is not below the destination root '{}'",
        format_path(parent),
        format_path(&self.root)
      )));
    }

    if !matches!(Path::new(name).components().next(), Some(Component::Normal(part)) if part == name) {
      return Err(XrfError::new_invalid_error(format!(
        "Refusing to write '{}': it is not one file name",
        format_path(name)
      )));
    }

    let path: PathBuf = parent.join(name);

    self.check_file_slot(&path)?;

    Ok(
      File::options()
        .read(false)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)?,
    )
  }

  fn create_parent<'a>(&self, relative: &'a HostRelativePath) -> XrfResult<(PathBuf, &'a OsStr)> {
    let Some(name) = relative.file_name() else {
      return Err(XrfError::new_invalid_error(format!(
        "Destination path '{relative}' names no file to write"
      )));
    };

    let parent: HostRelativePath = relative.parent().unwrap_or_default();

    Ok((self.create_directory(&parent)?, name))
  }

  /// The one step down, refused unless it is an ordinary name, whoever built the path.
  fn descend<'a>(component: Component<'a>, relative: &Path) -> XrfResult<&'a Path> {
    match component {
      Component::Normal(name) => Ok(Path::new(name)),
      _ => Err(XrfError::new_invalid_error(format!(
        "Destination path '{}' contains a component that would leave its root",
        format_path(relative)
      ))),
    }
  }

  /// Overwriting a file is ordinary; a link in its place is refused only when the policy says so.
  fn check_file_slot(&self, path: &Path) -> XrfResult {
    if self.link_policy == HostLinkPolicy::Follow {
      return Ok(());
    }

    match fs::symlink_metadata(path) {
      Ok(metadata) if is_link(&metadata) => Err(Self::new_link_error(path)),
      Ok(_) => Ok(()),
      Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
      Err(error) => Err(error.into()),
    }
  }

  /// Makes one component a directory, without walking through a link the policy refuses.
  fn create_component(&self, path: &Path) -> XrfResult {
    match self.read_metadata(path) {
      Ok(metadata) => self.ensure_directory(path, &metadata),
      Err(error) if error.kind() == ErrorKind::NotFound => match fs::create_dir(path) {
        Ok(()) => Ok(()),
        // Another worker of the same run got there first; what it left still has to be a directory.
        Err(error) if error.kind() == ErrorKind::AlreadyExists => {
          self.ensure_directory(path, &self.read_metadata(path)?)
        }
        Err(error) => Err(error.into()),
      },
      Err(error) => Err(error.into()),
    }
  }

  fn read_metadata(&self, path: &Path) -> IoResult<Metadata> {
    match self.link_policy {
      HostLinkPolicy::Follow => fs::metadata(path),
      HostLinkPolicy::Refuse => fs::symlink_metadata(path),
    }
  }

  fn ensure_directory(&self, path: &Path, metadata: &Metadata) -> XrfResult {
    if self.link_policy == HostLinkPolicy::Refuse && is_link(metadata) {
      return Err(Self::new_link_error(path));
    }

    if metadata.is_dir() {
      Ok(())
    } else {
      Err(XrfError::new_invalid_error(format!(
        "Cannot write into '{}' - it already exists and is not a directory",
        format_path(path)
      )))
    }
  }

  fn new_link_error(path: &Path) -> XrfError {
    XrfError::new_invalid_error(format!(
      "Cannot write through '{}' - it is a link, and this run refuses links below its destination",
      format_path(path)
    ))
  }
}

/// Whether an existing entry redirects a write somewhere the path does not spell.
///
/// Windows tests the reparse-point attribute: a junction or an unknown filter-driver tag redirects as a symlink does.
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
