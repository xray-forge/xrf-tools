use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;
use xrf_vfs::{XrayAsset, XrayMountMode, XrayMountPlan, XrayVfs};

use crate::patch::compare::ArchivePatchSide;
use crate::patch::config::ArchivePatchScope;
use crate::patch::world::archive_patch_entry_point::require_gamedata_entry_point;
use crate::patch::world::{ArchivePatchChecksum, ArchivePatchRole};

/// One side of a comparison: the roots it was given, mounted as the engine would see them.
///
/// Everything the comparison may ask about a side goes through here, so nothing downstream has to know whether an
/// entry came out of a volume or off the disk. That is what lets one comparison serve all four shapes of run.
pub(crate) struct ArchivePatchWorld {
  vfs: XrayVfs,
  roots: Vec<PathBuf>,
  role: ArchivePatchRole,
}

impl ArchivePatchWorld {
  /// Mount `roots` in engine order, where a later root overrides an earlier one.
  ///
  /// Engine order rather than priority order, because that is the order the thing being modelled is written in:
  /// `fsgame.ltx` declares `$arch_dir_patches$` after the content archives and `$game_data$` after all of them, and
  /// `CLocatorAPI::Register` overwrites on every hit, so the last declaration is what the game reads. The mount plan
  /// wants the opposite — [`XrayVfs`] takes the first mount that holds a path — so the list is reversed here, once,
  /// where the reason for it can be written down.
  ///
  /// # Errors
  ///
  /// Returns an invalid error when a root cannot be planned, when nothing mounted, or when a mounted volume declares
  /// an entry point other than the gamedata root.
  pub(crate) fn mount(roots: &[PathBuf], role: ArchivePatchRole) -> XrfResult<Self> {
    let world: Self = Self {
      vfs: XrayVfs::from_plan(&Self::plan(roots, role)?)?,
      roots: roots.to_vec(),
      role,
    };

    if world.vfs.is_empty() {
      return Err(XrfError::new_invalid_error(format!(
        "Nothing mounted from the {role} root(s): {}. A comparison needs two worlds, and an empty one would report \
         every file of the other as a difference.",
        world.describe_roots()
      )));
    }

    require_gamedata_entry_point(&world.vfs, role)?;

    Ok(world)
  }

  /// The entries in scope, by engine identity, in the order the VFS already sorts them.
  ///
  /// Sorted output is what lets the comparison walk both sides once rather than build a map of either, which at two
  /// mounted installations is the difference that decides the run's peak.
  pub(crate) fn list_scoped(&self, scope: &ArchivePatchScope) -> Vec<XrayAsset> {
    let mut entries: Vec<XrayAsset> = self.vfs.list_entries();

    entries.retain(|asset| scope.admits(asset.get_logical_path().as_str()));

    entries
  }

  /// What this side reports about one entry it holds.
  pub(crate) fn to_side(&self, asset: &XrayAsset) -> ArchivePatchSide {
    ArchivePatchSide {
      container: asset.get_container().clone(),
      size: self
        .vfs
        .read_size(asset.get_logical_path().as_str())
        .unwrap_or_default(),
    }
  }

  /// This side's checksum for one entry, taken from the source where it is free and computed where it is not.
  ///
  /// # Errors
  ///
  /// Returns the read error of a payload that had to be hashed.
  pub(crate) fn read_checksum(&self, name: &str) -> XrfResult<ArchivePatchChecksum> {
    match self.vfs.read_recorded_crc(name) {
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
    self.vfs.read_bytes(name)
  }

  /// The mounted world, for the packer reading payloads out of it.
  pub(crate) fn get_vfs(&self) -> &XrayVfs {
    &self.vfs
  }

  /// The root holding `path`, when this side mounted one that does.
  pub(crate) fn find_root_containing(&self, path: &Path) -> Option<&Path> {
    self
      .roots
      .iter()
      .find(|root| crate::path::is_inside_directory(path, root))
      .map(PathBuf::as_path)
  }

  pub(crate) const fn get_role(&self) -> ArchivePatchRole {
    self.role
  }

  pub(crate) fn describe_roots(&self) -> String {
    self
      .roots
      .iter()
      .map(|root| format!("'{}'", format_path(root)))
      .collect::<Vec<_>>()
      .join(", ")
  }

  /// The mount plan `roots` make, highest priority first.
  ///
  /// Walked in reverse so the last root named becomes the first mount planned, and each earlier one is layered
  /// *behind* what is already there — which is the same sentence as "a later root overrides an earlier one", written
  /// in the plan's own vocabulary.
  fn plan(roots: &[PathBuf], role: ArchivePatchRole) -> XrfResult<XrayMountPlan> {
    let mut plan: XrayMountPlan = XrayMountPlan::new();

    for root in roots.iter().rev() {
      let next: XrayMountPlan = XrayMountMode::Auto.plan(root).map_err(|error| {
        XrfError::new_invalid_error(format!("Cannot read the {role} root '{}': {error}", format_path(root)))
      })?;

      plan = plan.behind(next);
    }

    Ok(plan)
  }
}
