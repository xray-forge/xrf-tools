use std::path::Path;

use xrf_error::{XrfError, XrfResult};
use xrf_extension::{XrayExtension, XrayExtensionOf};
use xrf_utils::format_path;

/// How a configuration file of this crate is serialized.
///
/// Two spellings of one payload: the xrCompress dialect every existing configuration is written in, and JSON, which
/// automation can produce without knowing that dialect. Neither is the payload itself; this only says which encoding
/// a given file holds.
///
/// One type for packing and for patching, because the two answered this question with byte-identical enums differing
/// in a single noun of an error message. That noun is what a caller picks, through [`Self::of_pack_config`] and
/// [`Self::of_patch_config`]; everything else is decided once, here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveConfigFormat {
  /// LTX: `[include_folders]` for packing, `[include]` and `[ignore]` for patching, and in both a verbatim
  /// `[header]`.
  Ltx,
  /// The shape [`crate::ArchivePackConfigJson`] and [`crate::ArchivePatchConfigJson`] serialize to.
  Json,
}

impl ArchiveConfigFormat {
  /// Select the codec of a packing configuration from its path's extension.
  ///
  /// # Errors
  ///
  /// Returns an invalid error when the path carries no extension or one this does not serialize, naming both
  /// supported formats.
  pub fn of_pack_config<P: AsRef<Path>>(path: P) -> XrfResult<Self> {
    Self::of_config(path.as_ref(), "packing")
  }

  /// Select the codec of a patching configuration from its path's extension.
  ///
  /// # Errors
  ///
  /// Returns an invalid error when the path carries no extension or one this does not serialize, naming both
  /// supported formats.
  pub fn of_patch_config<P: AsRef<Path>>(path: P) -> XrfResult<Self> {
    Self::of_config(path.as_ref(), "patching")
  }

  /// The one selection, with `subject` naming the kind of configuration a refusal is about.
  ///
  /// Never guessed from contents: a configuration is a file a person named, so `pack.txt` holding LTX is a naming
  /// mistake to report rather than a format to detect.
  fn of_config(path: &Path, subject: &str) -> XrfResult<Self> {
    match XrayExtensionOf::of_path(path) {
      XrayExtensionOf::Known(XrayExtension::Ltx) => Ok(Self::Ltx),
      XrayExtensionOf::Known(XrayExtension::Json) => Ok(Self::Json),
      XrayExtensionOf::None => Err(XrfError::new_invalid_error(format!(
        "Cannot read '{}' as a {subject} configuration: it has no extension, and the format is taken from one. Name \
         it '.{}' or '.{}'.",
        format_path(path),
        XrayExtension::Ltx,
        XrayExtension::Json
      ))),
      // Every other spelling, whether or not the workspace models it: this serializes two of them.
      other => Err(XrfError::new_invalid_error(format!(
        "Cannot read '{}' as a {subject} configuration: '.{}' is not a supported format. Name it '.{}' or '.{}'.",
        format_path(path),
        other.as_str().unwrap_or_default().to_ascii_lowercase(),
        XrayExtension::Ltx,
        XrayExtension::Json
      ))),
    }
  }
}

#[cfg(test)]
mod tests {
  use xrf_error::XrfError;

  use super::ArchiveConfigFormat;

  #[test]
  fn selects_a_codec_from_the_extension_without_case() {
    for name in ["pack.ltx", "pack.LTX", "nested\\dir\\pack.Ltx"] {
      assert_eq!(
        ArchiveConfigFormat::of_pack_config(name).expect("ltx is recognized"),
        ArchiveConfigFormat::Ltx
      );
    }

    for name in ["patch.json", "patch.JSON"] {
      assert_eq!(
        ArchiveConfigFormat::of_patch_config(name).expect("json is recognized"),
        ArchiveConfigFormat::Json
      );
    }
  }

  #[test]
  fn each_configuration_is_refused_by_its_own_noun() {
    // The one thing the two collapsed enums did not share, and the thing the CLI contract describes.
    let packing: XrfError = ArchiveConfigFormat::of_pack_config("pack.txt").expect_err("txt is not a format");
    let patching: XrfError = ArchiveConfigFormat::of_patch_config("patch.txt").expect_err("txt is not a format");

    assert!(packing.to_string().contains("as a packing configuration"), "{packing}");
    assert!(
      patching.to_string().contains("as a patching configuration"),
      "{patching}"
    );
  }

  #[test]
  fn names_both_supported_formats_whichever_way_selection_failed() {
    let unsupported: XrfError = ArchiveConfigFormat::of_pack_config("pack.txt").expect_err("txt is not a format");
    let missing: XrfError = ArchiveConfigFormat::of_pack_config("pack").expect_err("an extension is required");

    assert!(unsupported.to_string().contains("'.txt' is not a supported format"));

    for error in [&unsupported, &missing] {
      assert!(error.to_string().contains("'.ltx'"), "{error}");
      assert!(error.to_string().contains("'.json'"), "{error}");
    }
  }

  #[test]
  fn names_a_refused_extension_in_one_case_whether_or_not_the_vocabulary_models_it() {
    // `XrayExtensionOf::as_str` answers the declared spelling for a modelled extension and the authored text for an
    // unmodelled one, so this used to report '.xml' for `pack.XML` and '.TXT' for `pack.TXT` - two cases in one
    // message, from a comparison that ignores case.
    let modelled: XrfError = ArchiveConfigFormat::of_pack_config("pack.XML").expect_err("xml is not a format");
    let unmodelled: XrfError = ArchiveConfigFormat::of_pack_config("pack.TXT").expect_err("txt is not a format");

    assert!(
      modelled.to_string().contains("'.xml' is not a supported format"),
      "{modelled}"
    );
    assert!(
      unmodelled.to_string().contains("'.txt' is not a supported format"),
      "{unmodelled}"
    );
  }

  #[test]
  fn a_dot_in_a_directory_name_is_not_the_format_of_the_file_inside_it() {
    // `Path::extension` agreed, but only because it splits the final component; the splitter says so on purpose.
    let error: XrfError = ArchiveConfigFormat::of_pack_config("out.json\\pack").expect_err("the file has no extension");

    assert!(error.to_string().contains("it has no extension"), "{error}");
  }
}
