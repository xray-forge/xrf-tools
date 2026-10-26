use std::ffi::OsStr;
use std::path::Path;

use xrf_extension::XrayExtension;

/// Which names are multi-language JSON translation sources.
///
/// One definition, because the build, the verifier, the initializer and the readers each decide this and three
/// spellings of it drifted once already.
pub(crate) struct TranslationSourceName;

impl TranslationSourceName {
  /// The stem of a JSON translation source filename, or nothing when the name is not one.
  ///
  /// The extension is matched without case, because the two halves of this crate see the same file spelled
  /// differently: a reader goes through the VFS, whose logical paths are lower case by definition, while the build and
  /// the verifier walk the host and see whatever the author typed. Matching exactly meant `ST_A.JSON` opened in the
  /// editor and was skipped by the build, with only an info line to say so. The stem keeps its original case, which
  /// is what the build writes its target as.
  ///
  /// Non-Unicode names are not translation sources, because their extension cannot be interpreted.
  pub(crate) fn parse_stem<T: AsRef<OsStr> + ?Sized>(file_name: &T) -> Option<&str> {
    let file_name: &str = file_name.as_ref().to_str()?;

    if !XrayExtension::Json.matches(file_name) {
      return None;
    }

    file_name.rsplit_once('.').map(|(stem, _)| stem)
  }

  /// Whether a name — a logical path's last component — is a source.
  ///
  /// The name form, for a reader working in the VFS domain where `Path::file_name` would be wrong: on Linux it answers
  /// the whole of a `\`-separated logical path.
  pub(crate) fn is_name(file_name: &str) -> bool {
    Self::parse_stem(file_name).is_some()
  }

  /// Whether a host path names a source.
  ///
  /// `Path::file_name` is correct here and only here: these are host paths from a directory walk, not engine
  /// identities.
  pub(crate) fn is_path(path: &Path) -> bool {
    path.file_name().and_then(Self::parse_stem).is_some()
  }
}
