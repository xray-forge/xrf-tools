use std::ffi::OsStr;
use std::path::Path;

use crate::json;

/// The stem of a JSON translation source filename, or nothing when the name is not one.
///
/// The extension is matched without case, because the two halves of this crate see the same file
/// spelled differently: a reader goes through the VFS, whose logical paths are lower case by
/// definition, while the build and the verifier walk the host and see whatever the author typed.
/// Matching exactly meant `ST_A.JSON` opened in the editor and was skipped by the build, with only an
/// info line to say so. The stem keeps its original case, which is what the build writes its target
/// as.
///
/// Non-Unicode names are not translation sources, because their extension cannot be interpreted.
pub(crate) fn parse_json_source_stem<T: AsRef<OsStr> + ?Sized>(file_name: &T) -> Option<&str> {
  let file_name: &str = file_name.as_ref().to_str()?;
  let (stem, extension): (&str, &str) = file_name.rsplit_once('.')?;

  extension.eq_ignore_ascii_case(json::FILE_EXTENSION).then_some(stem)
}

/// Whether a name — a logical path's last component — is a multi-language JSON source.
///
/// The name form, for a reader working in the VFS domain where `Path::file_name` would be wrong: on
/// Linux it answers the whole of a `\`-separated logical path.
pub(crate) fn is_json_source_name(file_name: &str) -> bool {
  parse_json_source_stem(file_name).is_some()
}

/// Whether a host path names a multi-language JSON source.
///
/// One definition, because the build, the verifier and the initializer each decide this and three
/// spellings of it drifted once already. `Path::file_name` is correct here and only here: these are
/// host paths from a directory walk, not engine identities.
pub(crate) fn is_json_source(path: &Path) -> bool {
  path.file_name().and_then(parse_json_source_stem).is_some()
}

#[cfg(test)]
mod tests {
  use std::ffi::{OsStr, OsString};

  use super::{is_json_source, parse_json_source_stem};

  #[test]
  fn parses_json_source_file_names() {
    assert_eq!(parse_json_source_stem(OsStr::new("st_items.json")), Some("st_items"));
    assert_eq!(parse_json_source_stem(&String::from("st_items.json")), Some("st_items"));
    assert_eq!(
      parse_json_source_stem(&OsString::from("nested.name.json")),
      Some("nested.name")
    );
  }

  #[test]
  fn refuses_everything_that_is_not_a_json_source() {
    // XML is a built artifact and a gamedata layout, never a source. A name that looks like the
    // retired language-suffixed form is just as much a non-source as any other extension.
    assert_eq!(parse_json_source_stem("dialogs.eng.xml"), None);
    assert_eq!(parse_json_source_stem("example.xml"), None);
    assert_eq!(parse_json_source_stem("dialogs.ukr.txt"), None);
    assert_eq!(parse_json_source_stem("no_extension"), None);
  }

  #[test]
  fn reads_a_host_name_in_any_case_the_author_typed_it() {
    // The VFS lower-cases logical paths, so a reader sees `st_a.json` where the build, walking the
    // host, sees `ST_A.JSON`. Matching exactly meant the editor opened a file the build then skipped,
    // with only an info line to say so.
    assert_eq!(parse_json_source_stem("ST_A.JSON"), Some("ST_A"));
    // The stem keeps the author's spelling, because that is what the build names its target after.
    assert_eq!(parse_json_source_stem("St_Items.Json"), Some("St_Items"));
    // Folding stops at the extension: an unrelated extension is still not a source.
    assert_eq!(parse_json_source_stem("dialogs.TXT"), None);
  }

  #[test]
  fn recognises_a_json_source_by_path() {
    assert!(is_json_source(std::path::Path::new("translations/st_items.json")));
    assert!(!is_json_source(std::path::Path::new("translations/st_ui.eng.xml")));
  }
}
