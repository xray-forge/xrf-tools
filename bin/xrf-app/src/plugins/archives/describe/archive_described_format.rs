use xrf_error::XrfResult;
use xrf_extension::{XrayExtension, XrayExtensionOf};
use xrf_vfs::XrayLogicalPath;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::archive_file_description::ArchiveFormatDescription;
use crate::plugins::archives::describe::omf::ArchiveOmfDescription;
use crate::plugins::archives::describe::particles::ArchiveParticlesDescription;
use crate::plugins::archives::describe::shaders::ArchiveShadersDescription;
use crate::plugins::archives::describe::thm::ArchiveThmDescription;

/// Which describer answers for an entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveDescribedFormat {
  /// A motion bank, `CKinematicsAnimated`'s partition and motions.
  Omf,
  /// The particle library, `particles.xr`.
  Particles,
  /// The compiled blender library, `shaders.xr`.
  Shaders,
  /// A texture descriptor, `ETextureThumbnail` wrapping `STextureParams`.
  Thm,
}

impl ArchiveDescribedFormat {
  /// The `.xr` libraries, by the file stem that tells one from another.
  ///
  /// The extension says only that a file is a chunked X-Ray library; six unrelated formats share it and the engine
  /// tells them apart by the name it loads each from. `gamemtl`, `lanims`, `senvironment` and `shaders_xrlc` are
  /// absent because nothing reads them yet, not because they are anything else.
  const LIBRARIES: [(&'static str, Self); 2] = [("particles", Self::Particles), ("shaders", Self::Shaders)];

  /// The describer for an entry name, or `None` when no describer claims it.
  pub fn of(name: &str) -> Option<Self> {
    match XrayExtensionOf::of(name).known()? {
      XrayExtension::Omf => Some(Self::Omf),
      XrayExtension::Thm => Some(Self::Thm),
      XrayExtension::Xr => Self::of_library(name),
      _ => None,
    }
  }

  /// Reads the entry this format claimed.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read, or when they are not the format this claimed them as.
  pub fn describe(self, source: &ArchiveDescribeSource, name: &str) -> XrfResult<ArchiveFormatDescription> {
    match self {
      Self::Omf => Ok(ArchiveFormatDescription::Omf {
        description: Box::new(ArchiveOmfDescription::read(source, name)?),
      }),
      Self::Particles => Ok(ArchiveFormatDescription::Particles {
        description: Box::new(ArchiveParticlesDescription::read(source, name)?),
      }),
      Self::Shaders => Ok(ArchiveFormatDescription::Shaders {
        description: Box::new(ArchiveShadersDescription::read(source, name)?),
      }),
      Self::Thm => Ok(ArchiveFormatDescription::Thm {
        description: Box::new(ArchiveThmDescription::read(source, name)?),
      }),
    }
  }

  /// Which `.xr` library an entry is, by its stem folded the way the engine addresses it.
  ///
  /// Through [`XrayLogicalPath`] rather than by slicing the name, so the case and separator rule is the one the name
  /// table already agrees on and is not restated here - which is the whole reason this decision is not the
  /// frontend's to make.
  fn of_library(name: &str) -> Option<Self> {
    let path: XrayLogicalPath = XrayLogicalPath::new(name).ok()?;
    let stem: &str = path.file_name().strip_suffix(".xr")?;

    Self::LIBRARIES
      .into_iter()
      .find_map(|(candidate, format)| (candidate == stem).then_some(format))
  }
}

#[cfg(test)]
mod tests {
  use super::ArchiveDescribedFormat;

  #[test]
  fn a_descriptor_is_claimed_however_its_extension_is_spelled() {
    assert_eq!(
      ArchiveDescribedFormat::of("textures\\act\\act_arm_1.thm"),
      Some(ArchiveDescribedFormat::Thm)
    );
    assert_eq!(
      ArchiveDescribedFormat::of("textures\\act\\act_arm_1.THM"),
      Some(ArchiveDescribedFormat::Thm)
    );
  }

  #[test]
  fn a_motion_bank_is_claimed_however_its_extension_is_spelled() {
    assert_eq!(
      ArchiveDescribedFormat::of("meshes\\actors\\stalker_animation.omf"),
      Some(ArchiveDescribedFormat::Omf)
    );
    assert_eq!(
      ArchiveDescribedFormat::of("meshes\\actors\\stalker_animation.OMF"),
      Some(ArchiveDescribedFormat::Omf)
    );
  }

  #[test]
  fn a_library_is_claimed_by_its_stem_rather_than_by_its_extension() {
    assert_eq!(
      ArchiveDescribedFormat::of("shaders.xr"),
      Some(ArchiveDescribedFormat::Shaders)
    );
    assert_eq!(
      ArchiveDescribedFormat::of("Particles.XR"),
      Some(ArchiveDescribedFormat::Particles)
    );
  }

  #[test]
  fn a_library_is_claimed_wherever_it_sits() {
    // The libraries ship at the root of a tree, but a name table addresses an entry by its whole path and an archive
    // is free to hold one deeper.
    assert_eq!(
      ArchiveDescribedFormat::of("mod\\overrides/Shaders.xr"),
      Some(ArchiveDescribedFormat::Shaders)
    );
  }

  #[test]
  fn a_library_nothing_reads_stays_unclaimed_though_it_shares_the_extension() {
    for name in ["gamemtl.xr", "lanims.xr", "senvironment.xr", "shaders_xrlc.xr"] {
      assert_eq!(ArchiveDescribedFormat::of(name), None, "'{name}' has no describer yet");
    }
  }

  #[test]
  fn a_stem_that_only_ends_in_a_library_name_is_not_one() {
    for name in ["my_shaders.xr", "shaders_backup.xr", "particles2.xr"] {
      assert_eq!(ArchiveDescribedFormat::of(name), None, "'{name}' is not a library");
    }
  }

  #[test]
  fn everything_without_a_describer_is_left_unclaimed() {
    for name in ["meshes\\actor.ogf", "textures\\act\\act_arm_1.dds", "level"] {
      assert_eq!(ArchiveDescribedFormat::of(name), None, "'{name}' has no describer yet");
    }
  }
}
