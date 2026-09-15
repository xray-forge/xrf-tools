use xrf_archive::ArchiveProject;
use xrf_error::{XrfError, XrfResult};
use xrf_vfs::{XrayAsset, XrayProbe};

use crate::core::assets::{AssetTextureDescriptor, AssetTextureShape};
use crate::plugins::archives::browse::ArchiveWorld;
use crate::plugins::archives::describe::archive_describe_scope::ArchiveDescribeScope;

/// The subject a description is taken from, as a describer sees it.
pub enum ArchiveDescribeSource<'a> {
  Volumes {
    project: &'a ArchiveProject,
  },
  World {
    world: &'a ArchiveWorld,
    probe: &'a XrayProbe<'a>,
  },
}

impl ArchiveDescribeSource<'_> {
  /// What this source's lookups search, for a surface wording an absence.
  pub fn get_scope(&self) -> ArchiveDescribeScope {
    match self {
      Self::Volumes { project } => ArchiveDescribeScope::Volumes {
        volumes: project.archives.len(),
      },
      Self::World { .. } => ArchiveDescribeScope::World,
    }
  }

  /// Unpacked bytes of an entry this source lists, or `None` when it lists no file under that name.
  ///
  /// A directory entry answers `None`: a volume records the directories it contains, and reading one as a file would
  /// describe an empty payload as a malformed descriptor.
  pub fn get_size_of(&self, name: &str) -> Option<u64> {
    match self {
      Self::Volumes { project } => project
        .files
        .get(name)
        .filter(|descriptor| !descriptor.is_directory)
        .map(|descriptor| u64::from(descriptor.size_real)),
      Self::World { world, .. } => world.find(name).map(|entry| entry.size_real),
    }
  }

  /// The name this source lists an engine path under, ready to select in the tree.
  pub fn find_entry(&self, path: &str) -> Option<String> {
    match self {
      Self::Volumes { project } => project
        .files
        .get(path)
        .filter(|descriptor| !descriptor.is_directory)
        .map(|descriptor| descriptor.name.to_string())
        .or_else(|| {
          project
            .files
            .values()
            .find(|descriptor| !descriptor.is_directory && is_same_logical_name(&descriptor.name, path))
            .map(|descriptor| descriptor.name.to_string())
        }),
      Self::World { world, .. } => world
        .find(path)
        .or_else(|| world.files.iter().find(|entry| is_same_logical_name(&entry.name, path)))
        .map(|entry| entry.name.clone()),
    }
  }

  /// Bytes of an entry this source lists.
  ///
  /// # Errors
  ///
  /// Returns an error when the source holds no such entry, or its bytes cannot be read out of the container.
  pub fn read_bytes(&self, name: &str) -> XrfResult<Vec<u8>> {
    match self {
      Self::Volumes { project } => project.read_file_bytes(name),
      Self::World { probe, .. } => probe.read_asset_bytes(&find_mounted_asset(probe, name)?),
    }
  }

  /// The shape a texture this source lists declares in its header, or `None` when it holds no such texture or its
  /// header will not parse.
  pub fn describe_texture(&self, name: &str) -> Option<AssetTextureShape> {
    match self {
      Self::Volumes { project } => AssetTextureShape::of_bytes(&project.read_file_bytes(name).ok()?),
      Self::World { probe, .. } => {
        AssetTextureDescriptor::describe(probe, &find_mounted_asset(probe, name).ok()?)?.shape
      }
    }
  }
}

/// The mounted asset behind an engine path.
///
/// # Errors
///
/// Returns an error when the path resolves to nothing in the mounted roots.
fn find_mounted_asset(probe: &XrayProbe, name: &str) -> XrfResult<XrayAsset> {
  probe
    .find(name)?
    .get_asset()
    .cloned()
    .ok_or_else(|| XrfError::new_not_found_error(format!("'{name}' resolves to nothing in the mounted roots")))
}

/// Whether an authored entry name is the same engine path as an already normalized one.
fn is_same_logical_name(authored: &str, normalized: &str) -> bool {
  authored.len() == normalized.len()
    && authored.bytes().zip(normalized.bytes()).all(|(candidate, expected)| {
      let candidate: u8 = if candidate == b'/' {
        b'\\'
      } else {
        candidate.to_ascii_lowercase()
      };

      candidate == expected
    })
}

#[cfg(test)]
mod tests {
  use super::is_same_logical_name;

  #[test]
  fn an_entry_matches_its_engine_identity_whatever_its_spelling() {
    let path: &str = "textures\\act\\act_arm_1.dds";

    assert!(is_same_logical_name(path, path));
    assert!(is_same_logical_name("textures\\Act\\ACT_arm_1.DDS", path));
    assert!(is_same_logical_name("textures/act/act_arm_1.dds", path));
  }

  #[test]
  fn a_different_entry_does_not_match() {
    let path: &str = "textures\\act\\act_arm_1.dds";

    assert!(!is_same_logical_name("textures\\act\\act_arm_2.dds", path));
    assert!(!is_same_logical_name("textures\\act\\act_arm_1.thm", path));
    assert!(!is_same_logical_name("textures\\act\\act_arm_1", path));
  }
}
