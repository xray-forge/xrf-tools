use xrf_extension::{XrayExtension, XrayExtensionOf};

/// Which describer answers for an entry, decided from its name alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveDescribedFormat {
  /// A texture descriptor, `ETextureThumbnail` wrapping `STextureParams`.
  Thm,
}

impl ArchiveDescribedFormat {
  /// The describer for an entry name, or `None` when no describer claims it.
  pub fn of(name: &str) -> Option<Self> {
    match XrayExtensionOf::of(name).known()? {
      XrayExtension::Thm => Some(Self::Thm),
      _ => None,
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
