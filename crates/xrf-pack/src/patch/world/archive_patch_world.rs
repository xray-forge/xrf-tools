use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;
use xrf_vfs::{XrayAsset, XrayMountMode, XraySearch, XraySearchPlan};

use crate::patch::compare::{ArchivePatchOrigins, ArchivePatchSide};
use crate::patch::config::ArchivePatchScope;
use crate::patch::world::{ArchivePatchChecksum, ArchivePatchEntryPoint, ArchivePatchRole};
use crate::path::is_inside_directory;

/// One comparison root mounted through the VFS, with archive and loose-file reads.
pub(crate) struct ArchivePatchWorld {
  search: XraySearch,
  root: PathBuf,
  role: ArchivePatchRole,
}

impl ArchivePatchWorld {
  /// Mounts an installation, archive directory, or loose tree with [`XrayMountMode::Auto`].
  ///
  /// Installations use `fsgame.ltx` mount order; later declarations win.
  ///
  /// # Errors
  ///
  /// Rejects unplannable or empty mounts and archive entry points outside gamedata. Propagates mount errors.
  pub(crate) fn mount(root: &Path, role: ArchivePatchRole) -> XrfResult<Self> {
    Self::of_plan(&Self::plan(root, role)?, root, role)
  }

  /// The search plan for a root, so a caller that needs both halves of one input can split it before opening anything.
  ///
  /// # Errors
  ///
  /// Returns an invalid error naming the role when the path cannot be planned.
  pub(crate) fn plan(root: &Path, role: ArchivePatchRole) -> XrfResult<XraySearchPlan> {
    XraySearchPlan::new()
      .with_root_mode(role.to_string(), root, XrayMountMode::Auto)
      .map_err(|error| {
        XrfError::new_invalid_error(format!("Cannot read the {role} root '{}': {error}", format_path(root)))
      })
  }

  /// Mounts an already-planned set of sources, named after `root` in anything it has to say.
  ///
  /// # Errors
  ///
  /// Rejects an empty mount set and archive entry points outside gamedata. Propagates mount errors.
  pub(crate) fn of_plan(plan: &XraySearchPlan, root: &Path, role: ArchivePatchRole) -> XrfResult<Self> {
    let world: Self = Self {
      search: plan.open()?,
      root: root.to_path_buf(),
      role,
    };

    if world.search.is_empty() {
      return Err(XrfError::new_invalid_error(format!(
        "Nothing mounted from the {role} root '{}'. A comparison needs two worlds, and an empty one would report \
         every file of the other as a difference.",
        world.describe_root()
      )));
    }

    ArchivePatchEntryPoint::require_gamedata(world.search.get_vfs(), role)?;

    Ok(world)
  }

  /// Returns entries in scope, sorted by engine identity.
  pub(crate) fn list_scoped(&self, scope: &ArchivePatchScope) -> Vec<XrayAsset> {
    let mut entries: Vec<XrayAsset> = self.search.list_files();

    entries.retain(|asset| scope.admits(asset.get_logical_path().as_str()));

    entries
  }

  /// What this side reports about one entry it holds.
  ///
  /// Takes the interner rather than owning one, so both worlds number their origins against the same table and an
  /// index means the same thing whichever side produced it.
  pub(crate) fn to_side(&self, asset: &XrayAsset, origins: &mut ArchivePatchOrigins) -> ArchivePatchSide {
    ArchivePatchSide {
      origin: origins.intern(asset.get_container()),
      size: self
        .search
        .read_size(asset.get_logical_path().as_str())
        .unwrap_or_default(),
    }
  }

  /// Returns the recorded archive CRC or hashes the loose payload.
  ///
  /// # Errors
  ///
  /// Propagates payload read errors.
  pub(crate) fn read_checksum(&self, name: &str) -> XrfResult<ArchivePatchChecksum> {
    match self.search.read_recorded_crc(name) {
      Some(crc) => Ok(ArchivePatchChecksum::Recorded(crc)),
      None => Ok(ArchivePatchChecksum::Hashed(crc32fast::hash(&self.read_bytes(name)?))),
    }
  }

  /// The payload itself.
  ///
  /// # Errors
  ///
  /// Returns the read error of the mounted entry.
  pub(crate) fn read_bytes(&self, name: &str) -> XrfResult<Vec<u8>> {
    self.search.read(name)
  }

  /// The search over this side, for the packer reading payloads out of it.
  pub(crate) fn get_search(&self) -> &XraySearch {
    &self.search
  }

  /// Whether this side's root holds `path`.
  pub(crate) fn contains(&self, path: &Path) -> bool {
    is_inside_directory(path, &self.root)
  }

  pub(crate) fn get_root(&self) -> &Path {
    &self.root
  }

  pub(crate) const fn get_role(&self) -> ArchivePatchRole {
    self.role
  }

  pub(crate) fn describe_root(&self) -> String {
    format_path(&self.root).to_string()
  }
}
