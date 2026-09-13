use std::path::Path;

use crate::file_extension::{get_file_extension, get_path_extension};
use crate::xray_extension::XrayExtension;

/// What a file name's extension turned out to be: one this workspace models, one it does not, or none at all.
///
/// The one door from a name to its extension, because both answers have a caller and asking twice is what let them
/// diverge. Logic wants the variant; a person-facing message wants the text of a spelling the vocabulary does not
/// cover, so that a viewer can say *why* it will not open a `.psd` instead of showing nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum XrayExtensionOf<'a> {
  Known(XrayExtension),
  /// The extension exactly as authored, case included, for a spelling the vocabulary does not hold.
  Unknown(&'a str),
  /// The name carries no extension. A level bundle's own `level` file is the engine's example of one.
  None,
}

impl<'a> XrayExtensionOf<'a> {
  /// Reads the extension of `name`, which may be an engine identity or a host file name.
  pub fn of(name: &'a str) -> Self {
    Self::of_split(get_file_extension(name))
  }

  /// Reads the extension of a host path, from its file name alone.
  pub fn of_path(path: &'a Path) -> Self {
    Self::of_split(get_path_extension(path))
  }

  /// What a splitter's answer turns into, shared so the two doors cannot classify differently.
  fn of_split(extension: Option<&'a str>) -> Self {
    match extension {
      Some(extension) => match XrayExtension::parse(extension) {
        Some(known) => Self::Known(known),
        None => Self::Unknown(extension),
      },
      None => Self::None,
    }
  }

  /// The extension this workspace models, for a caller that has nothing to say about the other two answers.
  pub fn known(self) -> Option<XrayExtension> {
    match self {
      Self::Known(extension) => Some(extension),
      Self::Unknown(_) | Self::None => None,
    }
  }

  /// The extension as text, for naming it to a person.
  ///
  /// A known extension answers with its declared spelling rather than with the authored case, because that is the one
  /// spelling every surface renders it as. An unknown one answers as authored — there is no canonical form to offer.
  pub fn as_str(self) -> Option<&'a str> {
    match self {
      Self::Known(extension) => Some(extension.as_str()),
      Self::Unknown(extension) => Some(extension),
      Self::None => None,
    }
  }
}

#[cfg(test)]
mod tests {
  use std::path::{Path, PathBuf};

  use xrf_test_utils::utils::build_non_unicode_file_name;

  use super::{XrayExtension, XrayExtensionOf};

  #[test]
  fn reads_a_modelled_extension_off_a_name() {
    assert_eq!(
      XrayExtensionOf::of("configs\\system.ltx"),
      XrayExtensionOf::Known(XrayExtension::Ltx)
    );
    assert_eq!(
      XrayExtensionOf::of("meshes/actors/stalker.ogf"),
      XrayExtensionOf::Known(XrayExtension::Ogf)
    );
    // The engine's leading-dot shader script, which `Path::extension` calls a hidden file.
    assert_eq!(
      XrayExtensionOf::of("shaders\\r1\\.s"),
      XrayExtensionOf::Known(XrayExtension::S)
    );
  }

  #[test]
  fn reads_a_modelled_extension_whatever_case_the_name_was_authored_in() {
    assert_eq!(
      XrayExtensionOf::of("TEXTURES\\WPN\\AK74.DDS"),
      XrayExtensionOf::Known(XrayExtension::Dds)
    );
  }

  #[test]
  fn keeps_the_text_of_a_spelling_it_does_not_model() {
    // What a viewer says instead of refusing silently.
    assert_eq!(
      XrayExtensionOf::of("textures\\wpn\\ak74.psd"),
      XrayExtensionOf::Unknown("psd")
    );
    // As authored, because an unknown spelling has no declared form to answer with.
    assert_eq!(
      XrayExtensionOf::of("textures\\wpn\\ak74.PSD"),
      XrayExtensionOf::Unknown("PSD")
    );
  }

  #[test]
  fn tells_an_unknown_extension_apart_from_no_extension() {
    // The distinction the whole type exists for: one is a file this cannot open, the other is a file whose kind is
    // read from its name instead, which is how a level bundle is recognized.
    assert_eq!(XrayExtensionOf::of("readme.txt"), XrayExtensionOf::Unknown("txt"));
    assert_eq!(XrayExtensionOf::of("levels\\l01_escape\\level"), XrayExtensionOf::None);
    assert_eq!(
      XrayExtensionOf::of("configs\\weapons.old\\readme"),
      XrayExtensionOf::None
    );
    assert_eq!(XrayExtensionOf::of(""), XrayExtensionOf::None);
  }

  #[test]
  fn reads_a_path_extension_off_the_file_name_alone() {
    assert_eq!(
      XrayExtensionOf::of_path(Path::new("configs/system.ltx")),
      XrayExtensionOf::Known(XrayExtension::Ltx)
    );
    assert_eq!(
      XrayExtensionOf::of_path(Path::new("preview.PSD")),
      XrayExtensionOf::Unknown("PSD")
    );
    // The directory carries the dot, so the file has no extension - which is a different refusal from an unsupported
    // one wherever a caller reports the two apart.
    assert_eq!(
      XrayExtensionOf::of_path(Path::new("out.json/pack")),
      XrayExtensionOf::None
    );
  }

  #[test]
  fn reads_a_path_extension_through_a_parent_directory_that_is_not_valid_text() {
    // Whole-path conversion answered `None` here, so a caller selecting a codec reported "it has no extension" for a
    // configuration that plainly had one.
    let path: PathBuf = PathBuf::from(build_non_unicode_file_name()).join("pack.ltx");

    assert!(path.to_str().is_none(), "the whole path has to be unreadable as text");
    assert_eq!(
      XrayExtensionOf::of_path(&path),
      XrayExtensionOf::Known(XrayExtension::Ltx)
    );
  }

  #[test]
  fn answers_the_variant_only_for_a_modelled_spelling() {
    assert_eq!(
      XrayExtensionOf::of("configs\\system.ltx").known(),
      Some(XrayExtension::Ltx)
    );
    assert_eq!(XrayExtensionOf::of("preview.psd").known(), None);
    assert_eq!(XrayExtensionOf::of("level").known(), None);
  }

  #[test]
  fn names_the_extension_of_both_kinds_of_answer() {
    assert_eq!(XrayExtensionOf::of("textures\\a.DDS").as_str(), Some("dds"));
    assert_eq!(XrayExtensionOf::of("preview.psd").as_str(), Some("psd"));
    assert_eq!(XrayExtensionOf::of("level").as_str(), None);
  }
}
