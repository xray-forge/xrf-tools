use xrf_error::XrfResult;
use xrf_extension::{XrayExtension, XrayExtensionOf};

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::archive_file_description::ArchiveFormatDescription;
use crate::plugins::archives::describe::omf::ArchiveOmfDescription;
use crate::plugins::archives::describe::thm::ArchiveThmDescription;

/// Which describer answers for an entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveDescribedFormat {
  /// A motion bank, `CKinematicsAnimated`'s partition and motions.
  Omf,
  /// A texture descriptor, `ETextureThumbnail` wrapping `STextureParams`.
  Thm,
}

impl ArchiveDescribedFormat {
  /// The describer for an entry name, or `None` when no describer claims it.
  pub fn of(name: &str) -> Option<Self> {
    match XrayExtensionOf::of(name).known()? {
      XrayExtension::Omf => Some(Self::Omf),
      XrayExtension::Thm => Some(Self::Thm),
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
      Self::Thm => Ok(ArchiveFormatDescription::Thm {
        description: Box::new(ArchiveThmDescription::read(source, name)?),
      }),
    }
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
  fn everything_without_a_describer_is_left_unclaimed() {
    for name in [
      "meshes\\actor.ogf",
      "gamemtl.xr",
      "textures\\act\\act_arm_1.dds",
      "level",
    ] {
      assert_eq!(ArchiveDescribedFormat::of(name), None, "'{name}' has no describer yet");
    }
  }
}
