use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use xrf_error::{XrfError, XrfResult};

use crate::pack::config::{ArchivePackConfig, ArchivePackMode, ArchiveVolumeExtension, default_header};
use crate::patch::config::{ArchivePatchScope, ArchivePatchShape};

/// Comparison roots, entry filters, and patch volume settings.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePatchConfig {
  /// What the patch is built against. Installations use `fsgame.ltx` mount order; later declarations win.
  pub input: PathBuf,
  /// The tree the patch delivers, or the loose half of [`Self::input`] when absent.
  ///
  /// Absent is the case nobody can spell by hand. An installation declares its volumes and its loose `gamedata\`
  /// together and the loose half wins, so comparing an installation with itself finds every loose file equal to
  /// itself; and naming `db\` mounts only the volumes sitting directly in it, because that is what a non-recursive
  /// declaration means. Splitting one mount plan by source kind is the only reading that answers "what have I
  /// actually changed in my game".
  pub target: Option<PathBuf>,
  pub destination: PathBuf,
  /// Base name of the volumes, which become `<name>.db0`, `<name>.db1` and so on.
  pub name: String,
  /// What the two sides are to each other, deciding whether a base-only entry is a removal or an untouched file.
  pub shape: ArchivePatchShape,
  /// Logical prefixes the comparison is restricted to, or the whole of both worlds when empty.
  pub include: Vec<String>,
  /// Logical prefixes dropped from the comparison, applied after [`Self::include`].
  pub ignore: Vec<String>,
  /// Extension patterns that keep a file out of the comparison, such as `*.txt`, matched with the dot.
  pub exclude_extensions: Vec<String>,
  /// Verbatim `[header]` text written as chunk 666, defaulting to the mountable gamedata header.
  pub header: Option<String>,
  pub mode: ArchivePackMode,
  pub max_volume_size: u64,
  pub is_with_oversized_volumes: bool,
  pub volume_extension: ArchiveVolumeExtension,
}

impl ArchivePatchConfig {
  /// An installation compared against its own loose tree, published to `destination` under `name`.
  pub fn new<I: AsRef<Path>, D: AsRef<Path>>(input: I, destination: D, name: &str) -> Self {
    Self {
      input: input.as_ref().into(),
      target: None,
      destination: destination.as_ref().into(),
      name: name.into(),
      shape: ArchivePatchShape::default(),
      include: Vec::new(),
      ignore: Vec::new(),
      exclude_extensions: Vec::new(),
      header: Some(default_header()),
      mode: ArchivePackMode::default(),
      max_volume_size: crate::VOLUME_SIZE_MAX,
      is_with_oversized_volumes: false,
      volume_extension: ArchiveVolumeExtension::default(),
    }
  }

  /// The same run delivering a tree of its own rather than the input's loose half.
  pub fn with_target<T: AsRef<Path>>(mut self, target: T) -> Self {
    self.target = Some(target.as_ref().into());

    self
  }

  /// The same run reading both sides as complete releases.
  pub const fn with_shape(mut self, shape: ArchivePatchShape) -> Self {
    self.shape = shape;

    self
  }

  /// Whether both sides come from splitting one mounted input.
  pub const fn is_splitting_input(&self) -> bool {
    self.target.is_none()
  }

  /// Builds volume settings for publication. Payloads come from the mounted target, not a source walk.
  pub(crate) fn to_publication(&self) -> ArchivePackConfig {
    ArchivePackConfig {
      header: self.header.clone(),
      is_with_oversized_volumes: self.is_with_oversized_volumes,
      max_volume_size: self.max_volume_size,
      mode: self.mode,
      volume_extension: self.volume_extension,
      ..ArchivePackConfig::new(self.get_target_root(), &self.destination, &self.name)
    }
  }

  /// Where carried payloads come from, which is the input itself when it supplies both sides.
  pub(crate) fn get_target_root(&self) -> &Path {
    self.target.as_deref().unwrap_or(&self.input)
  }

  /// What the comparison is allowed to look at.
  ///
  /// # Errors
  ///
  /// Returns an invalid error for a prefix that is not a name the engine can address.
  pub(crate) fn to_scope(&self) -> XrfResult<ArchivePatchScope> {
    ArchivePatchScope::new(&self.include, &self.ignore, &self.exclude_extensions)
  }

  /// Validates comparison and volume settings before mounting.
  ///
  /// # Errors
  ///
  /// Rejects a missing input, a named but empty target, release shape asked of a split input, invalid logical
  /// prefixes, and invalid packing settings.
  pub(crate) fn validate_for_patching(&self) -> XrfResult<()> {
    if self.input.as_os_str().is_empty() {
      return Err(XrfError::new_invalid_error(
        "A patch is built against something and no input was given",
      ));
    }

    if self.target.as_ref().is_some_and(|target| target.as_os_str().is_empty()) {
      return Err(XrfError::new_invalid_error(
        "A target was named but is empty. Leave it out to compare the input against its own loose tree.",
      ));
    }

    // Release shape asks what the target dropped, and one installation split in half has no answer: the volumes hold
    // the game the loose tree did not touch, which is not a deletion.
    if self.is_splitting_input() && self.shape.is_reporting_removals() {
      return Err(XrfError::new_invalid_error(
        "Release shape compares two complete releases, and one input split into its volumes and its loose tree is \
         not two releases. Name a target to compare releases, or drop the release shape.",
      ));
    }

    self.to_scope()?;
    self.to_publication().validate_for_packing()
  }
}
