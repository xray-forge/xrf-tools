use std::ffi::{OsStr, OsString};
use std::path::Path;

use crate::json::translation_source_name::TranslationSourceName;

#[test]
fn parses_json_source_file_names() {
  assert_eq!(
    TranslationSourceName::parse_stem(OsStr::new("st_items.json")),
    Some("st_items")
  );
  assert_eq!(
    TranslationSourceName::parse_stem(&String::from("st_items.json")),
    Some("st_items")
  );
  assert_eq!(
    TranslationSourceName::parse_stem(&OsString::from("nested.name.json")),
    Some("nested.name")
  );
}

#[test]
fn refuses_everything_that_is_not_a_json_source() {
  // XML is a built artifact and a gamedata layout, never a source. A name that looks like the retired
  // language-suffixed form is just as much a non-source as any other extension.
  assert_eq!(TranslationSourceName::parse_stem("dialogs.eng.xml"), None);
  assert_eq!(TranslationSourceName::parse_stem("example.xml"), None);
  assert_eq!(TranslationSourceName::parse_stem("dialogs.ukr.txt"), None);
  assert_eq!(TranslationSourceName::parse_stem("no_extension"), None);
}

#[test]
fn reads_a_host_name_in_any_case_the_author_typed_it() {
  // The VFS lower-cases logical paths, so a reader sees `st_a.json` where the build, walking the host, sees
  // `ST_A.JSON`. Matching exactly meant the editor opened a file the build then skipped, with only an info line to
  // say so.
  assert_eq!(TranslationSourceName::parse_stem("ST_A.JSON"), Some("ST_A"));
  // The stem keeps the author's spelling, because that is what the build names its target after.
  assert_eq!(TranslationSourceName::parse_stem("St_Items.Json"), Some("St_Items"));
  // Folding stops at the extension: an unrelated extension is still not a source.
  assert_eq!(TranslationSourceName::parse_stem("dialogs.TXT"), None);
}

#[test]
fn recognises_a_json_source_by_name_and_by_path() {
  assert!(TranslationSourceName::is_name("st_items.json"));
  assert!(TranslationSourceName::is_path(Path::new("translations/st_items.json")));
  assert!(!TranslationSourceName::is_path(Path::new("translations/st_ui.eng.xml")));
}
