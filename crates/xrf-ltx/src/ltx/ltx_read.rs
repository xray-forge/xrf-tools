use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_utils::{
  decode_bytes_to_string, encode_w1251_bytes_to_string, new_windows1251_encoder, read_as_string_from_w1251_encoded,
};
use xrf_vfs::{XrayLookupScope, XrayVfs};

use crate::dialect::{LtxDialect, LtxStandardDialect};
use crate::document::{LtxDocument, LtxParser};
use crate::ltx::{Ltx, LtxIncludeConvertor, LtxIncluded};
use crate::source::{LtxFilesystemSource, LtxVfsSource};

impl Ltx {
  /// Read LTX from a string.
  pub fn read_from_str(buf: &str) -> XrfResult<Self> {
    LtxStandardDialect::lower(&Self::read_document_from_str(buf)?)
  }

  /// Read one LTX file as the document it was written as, applying no dialect rule.
  pub fn read_document_from_str(buf: &str) -> XrfResult<LtxDocument> {
    LtxParser::new(buf.chars()).parse_document()
  }

  /// Read one LTX file as a document that also carries every line as authored.
  ///
  /// # Errors
  ///
  /// Returns an error when the contents will not parse.
  pub(crate) fn read_document_from_str_preserving_source(buf: &str) -> XrfResult<LtxDocument> {
    LtxParser::new_preserving_source(buf.chars()).parse_document()
  }

  /// Read LTX from a file as full parsed file, inject included files.
  pub(crate) fn read_from_file_included<P: AsRef<Path>>(filename: P) -> XrfResult<Self> {
    Self::read_from_path(filename)?.into_included()
  }

  /// Read LTX from a file under standard rules, injecting includes and flattening inheritance.
  ///
  /// Named for the dialect it applies because it applies one: a game config from a patched Anomaly install resolves
  /// differently, and silently. Use [`Self::read_from_file_with_dialect`] for anything a patch file could target, and
  /// this for XRF's own data files, which nothing patches.
  pub fn read_from_file_standard<P: AsRef<Path>>(filename: P) -> XrfResult<Self> {
    Self::read_from_path(filename)?.into_included()?.into_inherited()
  }

  /// Read LTX out of a mounted VFS, injecting includes resolved through it.
  ///
  /// This is how configs are read from an installation, where they sit inside `db\configs` volumes and have no filesystem
  /// path. Wildcard includes resolve by prefix enumeration rather than `read_dir`, and an include the VFS does not hold is
  /// nothing to merge rather than a failure - the same tolerance a not-yet-generated config gets on disk.
  ///
  /// `path` and `directory` on the result carry logical paths, not filesystem ones.
  ///
  /// # Errors
  ///
  /// Returns an error when the path is not in scope, its bytes are not valid Windows-1251, or the contents will not parse.
  pub(crate) fn read_from_vfs(vfs: &XrayVfs, scope: &XrayLookupScope, logical_path: &str) -> XrfResult<Self> {
    let source: LtxVfsSource = LtxVfsSource::new(vfs, scope);

    LtxIncludeConvertor::convert_with(source.read_ltx(logical_path)?, &source)
  }

  /// Read LTX out of a mounted VFS under standard rules, with includes injected and inheritance flattened.
  ///
  /// Resolve a game config through a project instead, which carries the chosen dialect; see
  /// `LtxProject::read_full_in_scope` for one outside the project's own prefix.
  ///
  /// # Errors
  ///
  /// Returns an error when reading fails, or when an inherited section cannot be resolved.
  pub(crate) fn read_from_vfs_standard(vfs: &XrayVfs, scope: &XrayLookupScope, logical_path: &str) -> XrfResult<Self> {
    Self::read_from_vfs(vfs, scope, logical_path)?.into_inherited()
  }

  /// Read from a file as generic ltx with LTX descriptor filled.
  pub fn read_from_path<P: AsRef<Path>>(filename: P) -> XrfResult<Self> {
    let mut ltx: Self = Self::read_from(&mut File::open(filename.as_ref())?)?;

    ltx.path = Some(PathBuf::from(filename.as_ref()));
    ltx.directory = filename.as_ref().parent().map(PathBuf::from);

    Ok(ltx)
  }

  /// Read a document from a reader, decoding strict Windows-1251.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not Windows-1251 or will not parse.
  pub(crate) fn read_document_from<R: Read>(reader: &mut R) -> XrfResult<LtxDocument> {
    Self::read_document_from_str(&read_as_string_from_w1251_encoded(reader)?)
  }

  /// Read a config from the filesystem, resolving it under `dialect`.
  ///
  /// What a command reading one named config should use rather than [`Self::read_from_file_standard`]: a patched
  /// install resolves differently, and the difference is silent. `path` is used as written, so a relative path
  /// resolves its includes relative to itself.
  ///
  /// # Errors
  ///
  /// Returns an error when the config cannot be read, or when the dialect refuses it.
  pub fn read_from_file_with_dialect<P: AsRef<Path>>(path: P, dialect: &dyn LtxDialect) -> XrfResult<Self> {
    Ok(
      dialect
        .resolve(&path.as_ref().to_string_lossy(), &LtxFilesystemSource)?
        .ltx,
    )
  }

  /// Read from a reader as generic ltx with LTX descriptor filled.
  pub(crate) fn read_from<R: Read>(reader: &mut R) -> XrfResult<Self> {
    Self::read_from_str(&read_as_string_from_w1251_encoded(reader)?)
  }
}

impl Ltx {
  /// Load include statements from a string.
  pub(crate) fn read_included_from_str(buf: &str) -> XrfResult<LtxIncluded> {
    Ok(
      Self::read_document_from_str(buf)?
        .list_included()
        .into_iter()
        .map(String::from)
        .collect(),
    )
  }

  /// Load include statements from a file.
  pub(crate) fn read_included_from_file<P: AsRef<Path>>(filename: P) -> XrfResult<LtxIncluded> {
    Self::read_included_from_str(&read_as_string_from_w1251_encoded(&mut File::open(filename.as_ref())?)?)
  }

  /// Load include statements from a config in a mounted VFS, without parsing its sections.
  ///
  /// Used when assembling a project: entry points are the files nothing else includes, which needs every file's include list
  /// and none of their contents.
  ///
  /// # Errors
  ///
  /// Returns an error when the path is not in scope, its bytes are not valid Windows-1251, or the contents will not parse.
  pub(crate) fn read_included_from_vfs(
    vfs: &XrayVfs,
    scope: &XrayLookupScope,
    logical_path: &str,
  ) -> XrfResult<LtxIncluded> {
    let bytes: Vec<u8> = vfs.scoped(scope).read_bytes(logical_path)?;

    Self::read_included_from_str(&decode_bytes_to_string(&bytes, new_windows1251_encoder())?)
  }
}

impl Ltx {
  /// Load formatted LTX as string from string.
  pub fn format_from_str(buf: &str) -> XrfResult<String> {
    Ok(Self::read_document_from_str(buf)?.to_formatted())
  }

  /// Load formatted LTX as string from file.
  pub(crate) fn format_from_file<P: AsRef<Path>>(filename: P) -> XrfResult<String> {
    Self::format_from_str(&read_as_string_from_w1251_encoded(&mut File::open(filename.as_ref())?)?)
  }

  /// Whether the given LTX bytes are already formatted canonically.
  ///
  /// Content is all a formatting verdict needs, so this answers for an archived config too — only rewriting one needs a
  /// file, which is why [`Self::format_file`] stays separate.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes cannot be decoded or parsed.
  pub(crate) fn is_formatted(bytes: &[u8]) -> XrfResult<bool> {
    let existing: String = encode_w1251_bytes_to_string(bytes)?;

    Ok(existing == Self::format_from_str(&existing)?)
  }
}

#[cfg(test)]
mod test {
  use std::path::PathBuf;

  use xrf_test_utils::utils::{build_relative_test_sample_file_path, write_generated_test_resource};

  use crate::ltx::Ltx;
  use crate::ltx::LtxIncluded;

  #[test]
  fn load_from_file() {
    let file_name: PathBuf = write_generated_test_resource(
      &build_relative_test_sample_file_path(file!(), "load_from_file.ltx"),
      b"[test]\nKey=Value\n",
    )
    .expect("generated LTX input to be written");

    let ltx: Ltx = Ltx::read_from_path(&file_name).unwrap();
    assert_eq!(ltx.get_from("test", "Key"), Some("Value"));
  }

  /// Format `input` through a file on disk and compare the result to `expected`, line by line.
  fn assert_formats(case: &str, input: &str, expected: &str) {
    let path: PathBuf = write_generated_test_resource(
      &build_relative_test_sample_file_path(file!(), &format!("{case}.ltx")),
      input,
    )
    .expect("generated LTX input to be written");

    assert_eq!(
      Ltx::format_from_file(path)
        .expect("input to format")
        .split("\r\n")
        .collect::<Vec<&str>>(),
      expected.split('\n').collect::<Vec<&str>>()
    );
  }

  const NOT_FORMATTED_INCLUDES_INHERITANCE_AND_BLANK_RUNS: &str = r#"
; comment1

#include "included1.ltx"
; comment1

#include "base\included2.ltx"
#include "base\included3.ltx"

[base_1]:  inherited1,    inherited2



  ; This is a comment

Key =    'Value'
Stuff= Other

;   This is a comment

[base_2]:   inherited1
"#;

  const FORMATTED_INCLUDES_INHERITANCE_AND_BLANK_RUNS: &str = r#"; comment1
#include "included1.ltx"
; comment1
#include "base\included2.ltx"
#include "base\included3.ltx"

[base_1]:inherited1,inherited2
; This is a comment
Key = 'Value'
Stuff = Other
; This is a comment

[base_2]:inherited1
"#;

  #[test]
  fn formats_includes_inheritance_and_blank_runs() {
    assert_formats(
      "formats_includes_inheritance_and_blank_runs",
      NOT_FORMATTED_INCLUDES_INHERITANCE_AND_BLANK_RUNS,
      FORMATTED_INCLUDES_INHERITANCE_AND_BLANK_RUNS,
    );
  }

  const NOT_FORMATTED_TRAILING_COMMENTS_AND_EMPTY_SECTIONS: &str = r#"#include "included1.ltx"
; comment1

#include "base\included2.ltx"
#include "base\included3.ltx"

[base_1]:  inherited1,  inherited2



  ; This is a comment

Key =    'Value'   ;    with comment
Stuff= Other;with comment

;   This is a comment
;   This is a comment
[base_2]:   inherited1
[base_3]
[base_4]
"#;

  const FORMATTED_TRAILING_COMMENTS_AND_EMPTY_SECTIONS: &str = r#"#include "included1.ltx"
; comment1
#include "base\included2.ltx"
#include "base\included3.ltx"

[base_1]:inherited1,inherited2
; This is a comment
Key = 'Value' ; with comment
Stuff = Other ; with comment
; This is a comment
; This is a comment

[base_2]:inherited1

[base_3]

[base_4]
"#;

  #[test]
  fn formats_trailing_comments_and_empty_sections() {
    assert_formats(
      "formats_trailing_comments_and_empty_sections",
      NOT_FORMATTED_TRAILING_COMMENTS_AND_EMPTY_SECTIONS,
      FORMATTED_TRAILING_COMMENTS_AND_EMPTY_SECTIONS,
    );
  }

  const NOT_FORMATTED_SECTION_HEADER_COMMENTS_AND_PATHS: &str = r#"#include "included1.ltx"
#include "included2.ltx"

; comment before section
[base_1]:  inherited1,  inherited2      ;  comment for section  ;  nested
Key = value ; with comment

;   This is a comment 1 ; nested comment
;   This is a comment 2 ; nested
[base_2]:   inherited1;   comment for base 2
field1 = 1
field2 = 2
[base_3];comment for base 3
field1 =   some\path\inside.ltx

[base_4]   ; comment for base 4
field1 = true
[base_5]:   ;;;; comment for base 5
field1 =
"#;

  const FORMATTED_SECTION_HEADER_COMMENTS_AND_PATHS: &str = r#"#include "included1.ltx"
#include "included2.ltx"
; comment before section

[base_1]:inherited1,inherited2 ; comment for section  ;  nested
Key = value ; with comment
; This is a comment 1 ; nested comment
; This is a comment 2 ; nested

[base_2]:inherited1 ; comment for base 2
field1 = 1
field2 = 2

[base_3] ; comment for base 3
field1 = some\path\inside.ltx

[base_4] ; comment for base 4
field1 = true

[base_5] ; ;;; comment for base 5
field1 =
"#;

  #[test]
  fn formats_section_header_comments_and_paths() {
    assert_formats(
      "formats_section_header_comments_and_paths",
      NOT_FORMATTED_SECTION_HEADER_COMMENTS_AND_PATHS,
      FORMATTED_SECTION_HEADER_COMMENTS_AND_PATHS,
    );
  }

  const NOT_FORMATTED_INCLUDE_COMMENTS_AND_BARE_VALUES: &str = r#"#include "included1.ltx"   ;      comment include 1
#include "included2.ltx";comment include 2

; comment before section
[base_1]:  inherited1,  inherited2      ;  comment for section
key = value ; with comment
value1
value2 ; comment
value3    ;   another comment

"#;

  const FORMATTED_INCLUDE_COMMENTS_AND_BARE_VALUES: &str = r#"#include "included1.ltx" ; comment include 1
#include "included2.ltx" ; comment include 2
; comment before section

[base_1]:inherited1,inherited2 ; comment for section
key = value ; with comment
value1
value2 ; comment
value3 ; another comment
"#;

  #[test]
  fn formats_include_comments_and_bare_values() {
    assert_formats(
      "formats_include_comments_and_bare_values",
      NOT_FORMATTED_INCLUDE_COMMENTS_AND_BARE_VALUES,
      FORMATTED_INCLUDE_COMMENTS_AND_BARE_VALUES,
    );
  }

  #[test]
  fn load_no_includes_from_file() {
    let file_name: PathBuf = write_generated_test_resource(
      &build_relative_test_sample_file_path(file!(), "load_no_includes.ltx"),
      b"[test]Key=Value\n",
    )
    .expect("generated LTX input to be written");

    let includes: LtxIncluded = Ltx::read_included_from_file(&file_name).unwrap();
    assert_eq!(includes, Vec::<String>::new());
  }

  #[test]
  fn load_few_includes_from_file() {
    let file_name: PathBuf = write_generated_test_resource(
      &build_relative_test_sample_file_path(file!(), "load_few_includes.ltx"),
      b"#include \"first.ltx\"\n;commented\n#include \"second.ltx\"",
    )
    .expect("generated LTX input to be written");

    let includes: LtxIncluded = Ltx::read_included_from_file(&file_name).unwrap();
    assert_eq!(includes, vec!("first.ltx", "second.ltx"));
  }

  #[test]
  fn invalid_codepoint() {
    use std::io::Cursor;

    let d = vec![
      10, 8, 68, 8, 61, 10, 126, 126, 61, 49, 10, 62, 8, 8, 61, 10, 91, 93, 93, 36, 91, 61, 10, 75, 91, 10, 10, 10, 61,
      92, 120, 68, 70, 70, 70, 70, 70, 126, 61, 10, 0, 0, 61, 10, 38, 46, 49, 61, 0, 39, 0, 0, 46, 92, 120, 46, 36, 91,
      91, 1, 0, 0, 16, 0, 0, 0, 0, 0, 0,
    ];
    let mut file = Cursor::new(d);
    assert!(Ltx::read_from(&mut file).is_err());
  }
}
