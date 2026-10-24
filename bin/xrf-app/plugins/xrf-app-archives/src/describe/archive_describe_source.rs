use xrf_app_core::assets::{AssetTextureDescriptor, AssetTextureShape};
use xrf_archive::{ArchiveFileDescriptor, ArchiveProject, ArchiveReadPolicy};
use xrf_chunk::ChunkReader;
use xrf_error::{XrfError, XrfResult};
use xrf_vfs::XrayAsset;

use crate::browse::{ArchiveNameIndex, ArchiveWorld};
use crate::describe::archive_describe_scope::ArchiveDescribeScope;
use crate::describe::archive_entry_reader::ArchiveEntryReader;

/// The subject a description is taken from, as a describer sees it.
pub enum ArchiveDescribeSource<'a> {
  Volumes {
    project: &'a ArchiveProject,
    names: &'a ArchiveNameIndex,
  },
  World {
    world: &'a ArchiveWorld,
  },
}

impl ArchiveDescribeSource<'_> {
  /// What this source's lookups search, for a surface wording an absence.
  pub fn get_scope(&self) -> ArchiveDescribeScope {
    match self {
      Self::Volumes { project, .. } => ArchiveDescribeScope::Volumes {
        volumes: project.archives.len(),
      },
      Self::World { .. } => ArchiveDescribeScope::World,
    }
  }

  /// What this source may be read under, which is the policy of the subject it is.
  pub fn get_read_policy(&self) -> &ArchiveReadPolicy {
    match self {
      Self::Volumes { project, .. } => &project.read_policy,
      Self::World { world } => &world.read_policy,
    }
  }

  /// Unpacked bytes of an entry this source lists, or `None` when it lists no file under that name.
  pub fn get_size_of(&self, name: &str) -> Option<u64> {
    self.find_file(name).map(|(_, size)| size)
  }

  /// The name this source lists an engine path under, ready to select in the tree.
  pub fn find_entry(&self, path: &str) -> Option<String> {
    self.find_file(path).map(|(name, _)| name.to_owned())
  }

  /// The one file this source lists for an engine path, however it is spelled, as the name it is listed under and its
  /// unpacked size: one lookup either way.
  ///
  /// Both public lookups answer from here, so describing an entry and resolving a reference to it can never disagree
  /// about which file that is.
  ///
  /// A directory answers `None`: a volume records the directories it contains, and reading one as a file would
  /// describe an empty payload as a malformed descriptor.
  fn find_file(&self, path: &str) -> Option<(&str, u64)> {
    match self {
      Self::Volumes { project, names } => names
        .find(project, path)
        .map(|descriptor: &ArchiveFileDescriptor| (descriptor.name.as_ref(), u64::from(descriptor.size_real))),
      Self::World { world } => world.find(path).map(|entry| (entry.name.as_str(), entry.size_real)),
    }
  }

  /// Opens an entry for reading, positionally where this source can.
  ///
  /// # Errors
  ///
  /// Returns an error when the source holds no such entry, when the file cannot be opened, or when its bytes cannot
  /// be read out of the container.
  pub fn open_entry(&self, name: &str) -> XrfResult<ArchiveEntryReader> {
    match self {
      Self::Volumes { project, .. } => Ok(ArchiveEntryReader::Held(ChunkReader::from_vec(
        project.read_file_bytes(name)?,
      )?)),
      Self::World { world } => {
        let asset: XrayAsset = world.find_asset(name)?;

        match asset.to_physical_path() {
          Some(path) => Ok(ArchiveEntryReader::Sliced(ChunkReader::from_path(path)?)),
          None => Ok(ArchiveEntryReader::Held(ChunkReader::from_vec(
            world.search.read_asset(&asset)?,
          )?)),
        }
      }
    }
  }

  /// Bytes of an entry this source lists.
  ///
  /// # Errors
  ///
  /// Returns an error when the source holds no such entry, or its bytes cannot be read out of the container.
  pub fn read_bytes(&self, name: &str) -> XrfResult<Vec<u8>> {
    match self {
      Self::Volumes { project, .. } => project.read_file_bytes(name),
      Self::World { world } => world.search.read_asset(&world.find_asset(name)?),
    }
  }

  /// The shape a texture this source lists declares in its header, or `None` when it holds no such texture or its
  /// header will not parse.
  ///
  /// A loose file costs only the header; an archived entry has no path to read a prefix of and so costs its payload,
  /// which is the standing price of describing anything inside a volume.
  pub fn get_texture_shape(&self, name: &str) -> Option<AssetTextureShape> {
    self.describe_texture(name).ok()?.shape
  }

  /// What a texture this source lists is: its size, and its header's facts where the header parses.
  ///
  /// # Errors
  ///
  /// Returns an error when the source holds no such entry, or its bytes cannot be reached at all.
  pub fn describe_texture(&self, name: &str) -> XrfResult<AssetTextureDescriptor> {
    match self {
      Self::Volumes { project, .. } => Ok(AssetTextureDescriptor::of_bytes(&project.read_file_bytes(name)?)),
      Self::World { world } => AssetTextureDescriptor::describe(&world.search, &world.find_asset(name)?)
        .ok_or_else(|| XrfError::new_asset_error(format!("'{name}' could not be read at all"))),
    }
  }
}
