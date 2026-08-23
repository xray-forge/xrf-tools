use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::path::Path;

use xrf_archive::ArchiveProject;
use xrf_error::{XrfError, XrfResult};

use crate::path::{is_component_prefix, normalize_logical};
use crate::source::xray_asset_source::label_from_path;
use crate::{XrayAssetContainer, XrayAssetSource, XraySourceKind};

/// Mounts an archive volume set as a read-only asset source.
///
/// Directory paths are scanned nonrecursively, matching `recurs = false` archive aliases and avoiding duplicate
/// subdirectory mounts.
///
/// [`ArchiveProject`] already merges a volume set into one name table with the later volume winning, which matches how the
/// engine registers them, so this adds only the logical-path keying a VFS lookup needs.
pub struct XrayArchiveSource {
  label: String,
  project: ArchiveProject,
  /// Normalized logical path to the key `project.files` stores.
  ///
  /// Archive headers keep names as authored, so the normalized form is derived once here rather than per lookup.
  entries: HashMap<String, String>,
}

impl XrayArchiveSource {
  /// Opens a volume set, or a single volume, at a path.
  pub fn read(path: impl AsRef<Path>) -> XrfResult<Self> {
    let path: &Path = path.as_ref();
    let project: ArchiveProject = ArchiveProject::new_shallow(path)?;

    let entries: HashMap<String, String> = project
      .files
      .iter()
      .filter(|(_, descriptor)| !descriptor.is_directory)
      .filter_map(|(name, _)| {
        normalize_logical(name)
          .inspect_err(|error| log::warn!("Skipping archive entry '{name}': {error}"))
          .ok()
          .map(|normalized| (normalized, name.clone()))
      })
      .collect();

    log::info!("Mounted {} archive entries from {}", entries.len(), path.display());

    Ok(Self {
      entries,
      label: label_from_path(path),
      project,
    })
  }

  /// The merged volume set behind this source, for consumers that need descriptor-level detail.
  pub fn get_project(&self) -> &ArchiveProject {
    &self.project
  }
}

/// Written by hand rather than derived, because a derived one would print the whole name table - 17,188 assets for
/// Anomaly's texture volumes alone. What identifies a mount is which volume set it is and how much it holds.
impl Debug for XrayArchiveSource {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("XrayArchiveSource")
      .field("label", &self.label)
      .field("root", &self.project.root)
      .field("entries", &self.entries.len())
      .finish()
  }
}

impl XrayAssetSource for XrayArchiveSource {
  fn get_label(&self) -> &str {
    &self.label
  }

  fn get_kind(&self) -> XraySourceKind {
    XraySourceKind::Archive
  }

  /// Always false. Writing into a volume is out of scope; a caller wanting to change an archived asset writes a loose
  /// override instead.
  fn is_writable(&self) -> bool {
    false
  }

  fn get_root_path(&self) -> &Path {
    &self.project.root
  }

  fn contains(&self, path: &str) -> bool {
    self.entries.contains_key(path)
  }

  fn locate(&self, path: &str) -> Option<XrayAssetContainer> {
    self.entries.contains_key(path).then(|| XrayAssetContainer::Archive {
      path: self.project.root.clone(),
    })
  }

  fn read(&self, path: &str) -> XrfResult<Vec<u8>> {
    let Some(name) = self.entries.get(path) else {
      // Absent, not unreadable: the distinction lets a caller fall back rather than fail.
      return Err(XrfError::new_not_found_error(format!(
        "no archive entry '{path}' in {}",
        self.label
      )));
    };

    self.project.read_file_bytes(name)
  }

  /// Answers from the volume's name table, so no entry is decompressed to learn its size.
  fn get_size(&self, path: &str) -> Option<u64> {
    self
      .entries
      .get(path)
      .and_then(|name| self.project.files.get(name))
      .map(|descriptor| u64::from(descriptor.size_real))
  }

  fn write(&self, path: &str, _bytes: &[u8]) -> XrfResult<()> {
    Err(XrfError::new_read_error(format!(
      "cannot write '{path}': archive '{}' is read only",
      self.label
    )))
  }

  /// Always fails. A volume cannot gain an entry; an override belongs in a loose mount in front of it.
  fn create(&self, path: &str, _bytes: &[u8]) -> XrfResult<()> {
    Err(XrfError::new_read_error(format!(
      "cannot create '{path}': archive '{}' is read only",
      self.label
    )))
  }

  fn list_entries<'a>(&'a self, prefix: Option<&'a str>) -> Box<dyn Iterator<Item = String> + 'a> {
    Box::new(
      self
        .entries
        .keys()
        .filter(move |path| prefix.is_none_or(|prefix| is_component_prefix(path, prefix)))
        .cloned(),
    )
  }
}
