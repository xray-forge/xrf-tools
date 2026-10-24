use std::path::Path;

use serde::Serialize;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::{AssetMountState, AssetTextureDescriptor};
use xrf_app_core::session::{SessionDerived, SessionSnapshot};
use xrf_archive::{ArchiveProject, ArchiveReadPolicy, ArchiveReadResult};
use xrf_archive_stats::ArchiveStatistics;
use xrf_pack::{
  ArchiveExtractDirectoryResult, ArchiveExtractOptions, ArchiveExtractResult, ArchiveUnpacker, XrayWorldExtractor,
};
use xrf_utils::error_to_string;
use xrf_vfs::{XrayArchiveSource, XrayInputRecord, XrayRoots, XraySearch};

use crate::browse::archive_name_index::ArchiveNameIndex;
use crate::browse::archive_override_report::ArchiveOverrideReport;
use crate::browse::archive_resolution::ArchiveResolution;
use crate::browse::archive_world::ArchiveWorld;
use crate::browse::archive_world_entry::ArchiveWorldEntry;
use crate::describe::{ArchiveDescribeSource, ArchiveFileDescription};

/// What the explorer has open: a set of `.db` volumes, or a whole mounted world.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ArchiveSubject {
  /// The volumes at one path, merged into a single name table.
  Volumes {
    project: ArchiveProject,
    /// The name table by engine identity, for references spelled otherwise than the volume lists them.
    #[serde(skip)]
    #[cfg_attr(feature = "typescript-bindings", specta(skip))]
    names: ArchiveNameIndex,
  },
  /// A game folder read as the engine mounts it, archives and loose tree together.
  World { world: ArchiveWorld },
}

impl ArchiveSubject {
  /// The volume set at `project`, its name table indexed on first need.
  pub fn of_volumes(project: ArchiveProject) -> Self {
    Self::Volumes {
      project,
      names: ArchiveNameIndex::default(),
    }
  }

  /// The world `search` reaches over `roots`, listed while recording what the listing read, so a commit to any of it
  /// lists the world again.
  pub fn read_world(search: XraySearch, roots: XrayRoots) -> SessionDerived<Self> {
    let inputs: XrayInputRecord = XrayInputRecord::new();
    let world: ArchiveWorld = ArchiveWorld::list(&search.clone().with_recording(&inputs), search, roots);

    SessionDerived::new(Self::World { world }, inputs)
  }

  /// The open world listed again over the roots it was opened over, once a commit moved what its listing read. A
  /// volume set records nothing it read, so nothing moves it.
  ///
  /// # Errors
  ///
  /// Returns an error when the roots cannot be searched, or when asked of a volume set.
  pub fn derive(current: &SessionSnapshot<Self>, assets: &AssetMountState) -> TauriResult<SessionDerived<Self>> {
    match &**current {
      Self::World { world } => Ok(Self::read_world(assets.search(&world.roots)?, world.roots.clone())),
      Self::Volumes { .. } => Err(String::from("An archive volume set does not follow commits")),
    }
  }

  /// What a describer reads this subject through.
  pub fn to_describe_source(&self) -> ArchiveDescribeSource<'_> {
    match self {
      Self::Volumes { project, names } => ArchiveDescribeSource::Volumes { project, names },
      Self::World { world } => ArchiveDescribeSource::World { world },
    }
  }

  /// What a viewer may read out of this subject, which is one policy for both.
  pub fn get_read_policy(&self) -> &ArchiveReadPolicy {
    match self {
      Self::Volumes { project, .. } => &project.read_policy,
      Self::World { world } => &world.read_policy,
    }
  }

  /// The volume set this subject is, or a refusal naming what it is instead.
  ///
  /// # Errors
  ///
  /// Returns a message naming the requirement when a world is open.
  pub fn require_volumes(&self) -> TauriResult<&ArchiveProject> {
    match self {
      Self::Volumes { project, .. } => Ok(project),
      Self::World { .. } => Err(String::from(
        "This command needs an archive volume set open, not a game folder",
      )),
    }
  }

  /// What this subject holds, broken down the ways a person asks about it.
  pub fn describe_statistics(&self) -> ArchiveStatistics {
    match self {
      Self::Volumes { project, .. } => ArchiveStatistics::of_volumes(project),
      Self::World { world } => ArchiveStatistics::of_world(&world.files, &world.mounts, &world.sources),
    }
  }

  /// What this subject's fold onto engine identities found, in one pass.
  pub fn describe_overrides(&self) -> ArchiveOverrideReport {
    match self {
      Self::Volumes { project, .. } => {
        let (overridden, unreachable) = XrayArchiveSource::describe_overrides_of(project);

        ArchiveOverrideReport {
          overridden: overridden.into_iter().map(ArchiveWorldEntry::from).collect(),
          unreachable,
        }
      }
      Self::World { world } => ArchiveOverrideReport {
        overridden: world
          .files
          .iter()
          .filter(|entry| !entry.shadowed.is_empty())
          .cloned()
          .collect(),
        unreachable: world.collisions.clone(),
      },
    }
  }

  /// Every source this subject searches, in the order it searches them.
  pub fn describe_resolution(&self) -> ArchiveResolution {
    match self {
      Self::Volumes { project, .. } => ArchiveResolution::of_volumes(project),
      Self::World { world } => ArchiveResolution::of_search(&world.search),
    }
  }

  /// Reads one file as text, subject to the viewer's read policy.
  ///
  /// # Errors
  ///
  /// Returns a message when the subject does not hold the name, the policy refuses it, or the bytes cannot be read or
  /// decoded.
  pub fn read_text(&self, name: &str) -> TauriResult<ArchiveReadResult> {
    match self {
      Self::Volumes { project, .. } => project.read_file_as_string(name),
      Self::World { world } => world.read_text(name),
    }
    .map_err(|error| format!("Failed to read '{name}': {error}"))
  }

  /// The stored bytes of one file.
  ///
  /// # Errors
  ///
  /// Returns a message when the subject does not hold the name, or its bytes cannot be read out of the container.
  pub fn read_bytes(&self, name: &str) -> TauriResult<Vec<u8>> {
    self
      .to_describe_source()
      .read_bytes(name)
      .map_err(|error| format!("Failed to read '{name}': {error}"))
  }

  /// Describes one file in words, for a format the viewer cannot draw.
  ///
  /// # Errors
  ///
  /// Returns a message when the subject does not hold the name, or a describer that claimed the entry could not read
  /// it. A format nothing describes is an answer rather than a failure.
  pub fn describe_file(&self, name: &str) -> TauriResult<ArchiveFileDescription> {
    ArchiveFileDescription::of(&self.to_describe_source(), name)
      .map_err(|error| format!("Failed to describe '{name}': {error}"))
  }

  /// Describes the shape of one texture without decoding it into a picture.
  ///
  /// # Errors
  ///
  /// Returns a message when the subject does not hold the name, or its bytes cannot be read at all.
  pub fn describe_texture(&self, name: &str) -> TauriResult<AssetTextureDescriptor> {
    self
      .to_describe_source()
      .describe_texture(name)
      .map_err(|error| format!("Failed to describe texture '{name}': {error}"))
  }

  /// Writes one file to a path the caller chose.
  ///
  /// # Errors
  ///
  /// Returns a message when the subject does not hold the name, or the bytes cannot be read or written.
  pub fn extract_file(&self, name: &str, destination: &Path) -> TauriResult<ArchiveExtractResult> {
    match self {
      Self::Volumes { project, .. } => ArchiveUnpacker::extract_file(project, name, destination),
      Self::World { world } => XrayWorldExtractor::extract_file(&world.search, name, destination),
    }
    .map_err(error_to_string)
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
    prefix: &str,
    destination: &Path,
    options: ArchiveExtractOptions,
  ) -> TauriResult<ArchiveExtractDirectoryResult> {
    match self {
      Self::Volumes { project, .. } => ArchiveUnpacker::extract_directory_opt(project, prefix, destination, options),
      Self::World { world } => XrayWorldExtractor::extract_directory_opt(&world.search, prefix, destination, options),
    }
    .map_err(error_to_string)
  }
}
