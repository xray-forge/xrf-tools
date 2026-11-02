use xrf_extension::XrayExtension;

use crate::path::XrayLogicalPath;

/// Where a kind of asset lives and what extension the engine loads it as.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct XrayAssetRules {
  /// Logical directory the engine resolves this kind under.
  pub directory: &'static str,
  /// Extension the engine loads.
  pub extension: XrayExtension,
  /// Extensions a reference may be authored with, which resolve to [`Self::extension`].
  pub authoring_extensions: &'static [XrayExtension],
}

impl XrayAssetRules {
  /// Converts a raw engine reference into the logical path below the kind's directory.
  pub fn to_logical_path(&self, reference: &str) -> String {
    if let Some((stem, extension)) = reference.rsplit_once('.') {
      if XrayExtension::parse(extension).is_some_and(|authored| self.authoring_extensions.contains(&authored)) {
        return format!("{stem}.{}", self.extension);
      }

      if self.extension.matches(reference) {
        return reference.to_string();
      }
    }

    format!("{reference}.{}", self.extension)
  }

  /// Converts a logical path below this kind's directory back into the reference the engine names it by.
  pub fn to_reference(&self, logical_path: &XrayLogicalPath) -> Option<String> {
    let below: &str = logical_path.strip_prefix(self.directory).ok()??;

    if !self.extension.matches(below) {
      return None;
    }

    // The extension matched, so the final dot is inside the final segment and this splits the name rather than a
    // directory holding one.
    let (reference, _) = below.rsplit_once('.')?;

    (!reference.is_empty()).then(|| reference.to_owned())
  }
}

#[cfg(test)]
mod tests {
  use super::XrayAssetRules;
  use crate::XrayAssetType;
  use crate::path::XrayLogicalPath;

  fn rules(asset_type: XrayAssetType) -> XrayAssetRules {
    asset_type.get_rules().expect("kind has a canonical home")
  }

  fn logical(path: &str) -> XrayLogicalPath {
    XrayLogicalPath::new(path).expect("valid logical path")
  }

  #[test]
  fn reads_a_reference_back_off_a_logical_path_in_the_kinds_home() {
    assert_eq!(
      rules(XrayAssetType::Dds)
        .to_reference(&logical("textures\\pfx\\smoke.dds"))
        .as_deref(),
      Some("pfx\\smoke")
    );
    assert_eq!(
      rules(XrayAssetType::Ogf)
        .to_reference(&logical("meshes\\actors\\stalker.ogf"))
        .as_deref(),
      Some("actors\\stalker")
    );
    // Round trip: the reference resolves back to the path it was read from.
    assert_eq!(
      rules(XrayAssetType::Dds).to_logical_path("pfx\\smoke"),
      "pfx\\smoke.dds"
    );
  }

  #[test]
  fn names_no_reference_for_a_path_outside_the_kinds_home_or_extension() {
    assert_eq!(
      rules(XrayAssetType::Dds).to_reference(&logical("levels\\l01\\lmap#0_1.dds")),
      None,
      "the directory has to match, not only the extension"
    );
    assert_eq!(
      rules(XrayAssetType::Dds).to_reference(&logical("textures_old\\a.dds")),
      None,
      "a directory sharing the prefix is not the home"
    );
    assert_eq!(
      rules(XrayAssetType::Dds).to_reference(&logical("textures\\pfx\\smoke.thm")),
      None,
      "a descriptor is not a texture, whatever directory it sits in"
    );
    assert_eq!(
      rules(XrayAssetType::Dds).to_reference(&logical("textures\\.dds")),
      None,
      "an extension alone names nothing"
    );
  }

  #[test]
  fn replaces_an_authoring_extension_with_the_loaded_one() {
    assert_eq!(
      rules(XrayAssetType::Dds).to_logical_path("pfx\\smoke.tga"),
      "pfx\\smoke.dds"
    );
    assert_eq!(
      rules(XrayAssetType::Dds).to_logical_path("pfx\\smoke.TGA"),
      "pfx\\smoke.dds"
    );
    assert_eq!(
      rules(XrayAssetType::Dds).to_logical_path("pfx\\smoke.bmp"),
      "pfx\\smoke.dds"
    );
  }

  #[test]
  fn appends_the_extension_when_a_reference_omits_it() {
    assert_eq!(
      rules(XrayAssetType::Dds).to_logical_path("pfx\\smoke"),
      "pfx\\smoke.dds"
    );
    assert_eq!(
      rules(XrayAssetType::Ogg).to_logical_path("weapons\\ak74_shot"),
      "weapons\\ak74_shot.ogg"
    );
    assert_eq!(
      rules(XrayAssetType::Ogg).to_logical_path("weapons\\ak74_shot.OGG"),
      "weapons\\ak74_shot.OGG",
      "an uppercase extension is not doubled"
    );
    assert_eq!(
      rules(XrayAssetType::Ogf).to_logical_path("actors\\stalker"),
      "actors\\stalker.ogf"
    );
  }

  #[test]
  fn leaves_an_already_loaded_extension_alone_whatever_its_case() {
    // Appending a second extension resolves to nothing, which is how mesh references silently failed.
    assert_eq!(
      rules(XrayAssetType::Ogf).to_logical_path("actors\\stalker.OGF"),
      "actors\\stalker.OGF"
    );
    assert_eq!(
      rules(XrayAssetType::Dds).to_logical_path("pfx\\smoke.dds"),
      "pfx\\smoke.dds"
    );
  }

  #[test]
  fn treats_an_unknown_extension_as_part_of_the_name() {
    // Not a gap in the authoring list: a reference is a name rather than a filename, so an extension the row does not
    // claim stays part of it and the loaded one is appended.
    assert_eq!(
      rules(XrayAssetType::Dds).to_logical_path("pfx\\smoke.png"),
      "pfx\\smoke.png.dds"
    );
  }
}
