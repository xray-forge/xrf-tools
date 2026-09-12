use std::path::Path;

use serde::Serialize;
use xrf_archive::{ArchiveProject, ArchiveReadPolicy, ArchiveReadResult};
use xrf_pack::{
  ArchiveExtractDirectoryResult, ArchiveExtractOptions, ArchiveExtractResult, ArchiveUnpacker, XrayWorldExtractor,
};
use xrf_vfs::{XrayArchiveSource, XrayPathCollision};

use crate::core::assets::AssetMountState;
use crate::core::types::TauriResult;
use crate::plugins::archives::browse::archive_world::ArchiveWorld;

/// What the explorer has open: a set of `.db` volumes, or a whole mounted world.
///
/// The two are not folded into one shape. A volume set can say which volume an entry sits in, at what offset, with
/// which recorded CRC; a world can say which copy of an engine path wins and what that decision hides. A single
/// descriptor covering both would have a loose file claiming a volume position, which is the fiction this split
/// exists to avoid.
///
/// Every difference between them is answered here, so a command stays an adapter and a reader asking how browsing an
/// installation differs from browsing a volume set opens one file.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ArchiveSubject {
  /// The volumes at one path, merged into a single name table.
  Volumes { project: ArchiveProject },
  /// A game folder read as the engine mounts it, archives and loose tree together.
  World { world: ArchiveWorld },
}

impl ArchiveSubject {
  /// What a viewer may read out of this subject, which is one policy for both.
  pub fn get_read_policy(&self) -> &ArchiveReadPolicy {
    match self {
      Self::Volumes { project } => &project.read_policy,
      Self::World { world } => &world.read_policy,
    }
  }

  /// The volume set this subject is, or a refusal naming what it is instead.
  ///
  /// The one place a name-table command narrows the subject, so each states the requirement by calling this rather
  /// than by matching the enum and inventing its own wording.
  ///
  /// # Errors
  ///
  /// Returns a message naming the requirement when a world is open.
  pub fn require_volumes(&self) -> TauriResult<&ArchiveProject> {
    match self {
      Self::Volumes { project } => Ok(project),
      Self::World { .. } => Err(String::from(
        "This command needs an archive volume set open, not a game folder",
      )),
    }
  }

  /// Entries this subject holds that no engine lookup can reach.
  ///
  /// A world folded them while its mounts were walked; a volume set folds its merged name table here, which an
  /// installation sizes rather than a gesture — so callers run this off the executor.
  pub fn list_collisions(&self) -> Vec<XrayPathCollision> {
    match self {
      Self::Volumes { project } => XrayArchiveSource::list_collisions_of(project),
      Self::World { world } => world.collisions.clone(),
    }
  }

  /// Reads one file as text, subject to the viewer's read policy.
  ///
  /// # Errors
  ///
  /// Returns a message when the subject does not hold the path, the policy refuses it, or the bytes cannot be read or
  /// decoded.
  pub fn read_text(&self, assets: &AssetMountState, name: &str) -> TauriResult<ArchiveReadResult> {
    match self {
      Self::Volumes { project } => project.read_file_as_string(name),
      Self::World { world } => assets.with_probe(&world.roots, |probe| world.read_text(probe, name))?,
    }
    .map_err(|error| format!("Failed to read '{name}': {error}"))
  }

  /// Writes one file to a path the caller chose.
  ///
  /// # Errors
  ///
  /// Returns a message when the subject does not hold the path, or the bytes cannot be read or written.
  pub fn extract_file(
    &self,
    assets: &AssetMountState,
    name: &str,
    destination: &str,
  ) -> TauriResult<ArchiveExtractResult> {
    match self {
      Self::Volumes { project } => ArchiveUnpacker::extract_file(project, name, destination),
      Self::World { world } => assets.with_probe(&world.roots, |probe| {
        XrayWorldExtractor::extract_file(probe, name, destination)
      })?,
    }
    .map_err(|error| error.to_string())
  }

  /// Writes everything under one directory into a destination root, as `options` asks.
  ///
  /// An empty prefix means the whole tree, so this is a full unpack in everything but name.
  ///
  /// # Errors
  ///
  /// Returns a message when nothing lies under the prefix, or an entry cannot be read or written.
  pub fn extract_directory(
    &self,
    assets: &AssetMountState,
    prefix: &str,
    destination: &Path,
    options: ArchiveExtractOptions,
  ) -> TauriResult<ArchiveExtractDirectoryResult> {
    match self {
      Self::Volumes { project } => ArchiveUnpacker::extract_directory_opt(project, prefix, destination, options),
      Self::World { world } => assets.with_probe(&world.roots, |probe| {
        XrayWorldExtractor::extract_directory_opt(probe, prefix, destination, options)
      })?,
    }
    .map_err(|error| error.to_string())
  }
}
