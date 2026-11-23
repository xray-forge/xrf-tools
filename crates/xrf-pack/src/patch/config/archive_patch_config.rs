use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use xrf_error::{XrfError, XrfResult};

use crate::pack::config::{ArchivePackConfig, ArchiveVolumeSettings};
use crate::patch::config::ArchivePatchScope;

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
  /// Logical prefixes the comparison is restricted to, or the whole of both worlds when empty.
  pub include: Vec<String>,
  /// Logical prefixes dropped from the comparison, applied after [`Self::include`].
  pub ignore: Vec<String>,
  /// Extension patterns that keep a file out of the comparison, such as `*.txt`, matched with the dot.
  pub exclude_extensions: Vec<String>,
  /// How the published volumes are written, as a pack writes them.
  #[serde(flatten)]
  pub volume_settings: ArchiveVolumeSettings,
}

impl ArchivePatchConfig {
  /// An installation compared against its own loose tree, published to `destination` under `name`.
  pub fn new<I: AsRef<Path>, D: AsRef<Path>>(input: I, destination: D, name: &str) -> Self {
    Self {
      input: input.as_ref().into(),
      target: None,
      destination: destination.as_ref().into(),
      name: name.into(),
      include: Vec::new(),
      ignore: Vec::new(),
      exclude_extensions: Vec::new(),
      volume_settings: ArchiveVolumeSettings::default(),
    }
  }

  /// The same run delivering a tree of its own rather than the input's loose half.
  pub fn with_target<T: AsRef<Path>>(mut self, target: T) -> Self {
    self.target = Some(target.as_ref().into());

    self
  }

  /// Whether both sides come from splitting one mounted input.
  pub const fn is_splitting_input(&self) -> bool {
    self.target.is_none()
  }

  /// Builds volume settings for publication. Payloads come from the mounted target, not a source walk.
  pub(crate) fn to_publication(&self) -> ArchivePackConfig {
    ArchivePackConfig {
      volume_settings: self.volume_settings.clone(),
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
  /// Rejects a missing input, a named but empty target, invalid logical prefixes, and invalid packing settings.
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

    self.to_scope()?;
    self.to_publication().validate_for_packing()
  }
}
