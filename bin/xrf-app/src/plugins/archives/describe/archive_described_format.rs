use xrf_error::XrfResult;
use xrf_extension::{XrayExtension, XrayExtensionOf};
use xrf_vfs::XrayLogicalPath;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::archive_file_description::ArchiveFormatDescription;
use crate::plugins::archives::describe::level::ArchiveLevelDescription;
use crate::plugins::archives::describe::omf::ArchiveOmfDescription;
use crate::plugins::archives::describe::particles::ArchiveParticlesDescription;
use crate::plugins::archives::describe::shaders::ArchiveShadersDescription;
use crate::plugins::archives::describe::spawn::ArchiveSpawnDescription;
use crate::plugins::archives::describe::thm::ArchiveThmDescription;

/// Which describer answers for an entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveDescribedFormat {
  /// A compiled level bundle, `level`.
  Level,
  /// A motion bank, `CKinematicsAnimated`'s partition and motions.
  Omf,
  /// The particle library, `particles.xr`.
  Particles,
  /// The compiled blender library, `shaders.xr`.
  Shaders,
  /// A spawn set, `all.spawn` and whatever else carries its header.
  Spawn,
  /// A texture descriptor, `ETextureThumbnail` wrapping `STextureParams`.
  Thm,
}

impl ArchiveDescribedFormat {
  /// Files the engine loads by a name of their own rather than by a kind, folded as engine paths.
  ///
  /// Two different reasons land here. `.xr` says only that a file is a chunked X-Ray library, and six unrelated
  /// formats share it; `level` has no extension at all. Either way the name the engine loads the file under is the
  /// whole of the answer. `gamemtl.xr`, `lanims.xr`, `senvironment.xr` and `shaders_xrlc.xr` are absent because
  /// nothing reads them yet, not because they are anything else.
  const NAMED: [(&'static str, Self); 3] = [
    ("level", Self::Level),
    ("particles.xr", Self::Particles),
    ("shaders.xr", Self::Shaders),
  ];

  /// The describer for an entry name, or `None` when no describer claims it.
  pub fn of(name: &str) -> Option<Self> {
    match XrayExtensionOf::of(name).known() {
      Some(XrayExtension::Omf) => Some(Self::Omf),
      Some(XrayExtension::Spawn) => Some(Self::Spawn),
      Some(XrayExtension::Thm) => Some(Self::Thm),
      // An extension that names a container rather than a format, and a name carrying none at all, ask the same
      // question: which file is this, by the name the engine loads it under.
      Some(XrayExtension::Xr) | None => Self::of_named(name),
      Some(_) => None,
    }
  }

  /// Whether this describer reads by seeking rather than by holding the entry, and so answers to no size ceiling.
  pub const fn reads_by_seeking(self) -> bool {
    matches!(self, Self::Spawn)
  }

  /// Reads the entry this format claimed, or answers `None` where the claim does not hold after all.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read, or when they are not the format this claimed them as.
  pub fn describe(self, source: &ArchiveDescribeSource, name: &str) -> XrfResult<Option<ArchiveFormatDescription>> {
    Ok(match self {
      Self::Level => Some(ArchiveFormatDescription::Level {
        description: Box::new(ArchiveLevelDescription::read(source, name)?),
      }),
      Self::Omf => Some(ArchiveFormatDescription::Omf {
        description: Box::new(ArchiveOmfDescription::read(source, name)?),
      }),
      Self::Particles => Some(ArchiveFormatDescription::Particles {
        description: Box::new(ArchiveParticlesDescription::read(source, name)?),
      }),
      Self::Shaders => Some(ArchiveFormatDescription::Shaders {
        description: Box::new(ArchiveShadersDescription::read(source, name)?),
      }),
      Self::Spawn => ArchiveSpawnDescription::read(source, name)?.map(|description| ArchiveFormatDescription::Spawn {
        description: Box::new(description),
      }),
      Self::Thm => Some(ArchiveFormatDescription::Thm {
        description: Box::new(ArchiveThmDescription::read(source, name)?),
      }),
    })
  }

  /// Which named file an entry is, folded the way the engine addresses it.
  ///
  /// Through [`XrayLogicalPath`] rather than by slicing the name, so the case and separator rule is the one the name
  /// table already agrees on and is not restated here - which is the whole reason this decision is not the
  /// frontend's to make.
  fn of_named(name: &str) -> Option<Self> {
    let path: XrayLogicalPath = XrayLogicalPath::new(name).ok()?;
    let file: &str = path.file_name();

    Self::NAMED
      .into_iter()
      .find_map(|(candidate, format)| (candidate == file).then_some(format))
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
  fn a_bundle_is_claimed_though_its_name_carries_no_extension() {
    assert_eq!(
      ArchiveDescribedFormat::of("levels\\l01_escape\\level"),
      Some(ArchiveDescribedFormat::Level)
    );
    assert_eq!(
      ArchiveDescribedFormat::of("Levels/L01_Escape/LEVEL"),
      Some(ArchiveDescribedFormat::Level)
    );
  }

  #[test]
  fn a_name_that_only_begins_with_a_named_file_is_not_one() {
    for name in [
      "levels\\l01_escape\\level.ai",
      "levels\\l01_escape\\level_lods",
      "mylevel",
    ] {
      assert_eq!(ArchiveDescribedFormat::of(name), None, "'{name}' is not a bundle");
    }
  }

  #[test]
  fn everything_without_a_describer_is_left_unclaimed() {
    for name in [
      "meshes\\actor.ogf",
      "textures\\act\\act_arm_1.dds",
      "levels\\l01\\level.geom",
    ] {
      assert_eq!(ArchiveDescribedFormat::of(name), None, "'{name}' has no describer yet");
    }
  }
}
