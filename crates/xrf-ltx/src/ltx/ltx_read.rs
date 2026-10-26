use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_utils::{decode_w1251_bytes_to_string, read_as_string_from_w1251_encoded};
use xrf_vfs::{XrayLogicalPath, XraySearch};

use crate::dialect::{LtxDialect, LtxResolveRequest, LtxStandardDialect};
use crate::document::{LtxDocument, LtxParser};
use crate::ltx::Ltx;
use crate::source::{LtxFilesystemSource, LtxVfsSource};

impl Ltx {
  /// Read one document from a string under standard rules, without reading anything it includes.
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
  pub fn read_document_from_str_preserving_source(buf: &str) -> XrfResult<LtxDocument> {
    LtxParser::new_preserving_source(buf.chars()).parse_document()
  }

  /// Read a config from the filesystem under standard rules, merging its includes and flattening inheritance.
  ///
  /// # Errors
  ///
  /// Returns an error when the config or a file it includes cannot be read, or when standard rules refuse the tree.
  pub fn read_from_file_standard<P: AsRef<Path>>(path: P) -> XrfResult<Self> {
    Self::read_from_file_with_dialect(path, &LtxStandardDialect)
  }

  /// Read a config through a search under standard rules, merging its includes and flattening inheritance.
  ///
  /// # Errors
  ///
  /// Returns an error when the search does not reach the config or a file it includes, its bytes are not valid
  /// Windows-1251, or standard rules refuse the tree.
  pub(crate) fn read_from_search_standard(search: &XraySearch, logical_path: &str) -> XrfResult<Self> {
    // Normalized first, so the location the config records is its engine identity rather than the spelling asked for.
    let logical_path: XrayLogicalPath = XrayLogicalPath::new(logical_path)?;

    Ok(
      LtxStandardDialect
        .resolve(
          logical_path.as_str(),
          &LtxVfsSource::new(search.clone()),
          LtxResolveRequest::plain(),
        )?
        .ltx,
    )
  }

  /// Read one config file under standard rules, recording where it was read from, without reading what it includes.
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
  /// # Errors
  ///
  /// Returns an error when the config cannot be read, or when the dialect refuses it.
  pub fn read_from_file_with_dialect<P: AsRef<Path>>(path: P, dialect: &dyn LtxDialect) -> XrfResult<Self> {
    Ok(
      dialect
        .resolve(
          &path.as_ref().to_string_lossy(),
          &LtxFilesystemSource,
          LtxResolveRequest::plain(),
        )?
        .ltx,
    )
  }

  /// Read one document from a reader under standard rules, decoding strict Windows-1251.
  pub(crate) fn read_from<R: Read>(reader: &mut R) -> XrfResult<Self> {
    Self::read_from_str(&read_as_string_from_w1251_encoded(reader)?)
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
  /// # Errors
  ///
  /// Returns an error when the bytes cannot be decoded or parsed.
  pub(crate) fn is_formatted(bytes: &[u8]) -> XrfResult<bool> {
    let existing: String = decode_w1251_bytes_to_string(bytes)?;

    Ok(existing == Self::format_from_str(&existing)?)
  }
}

#[cfg(test)]
mod test {
  use std::path::PathBuf;

  use xrf_test_utils::GeneratedResources;

  use crate::ltx::Ltx;

  #[test]
  fn load_from_file() {
    let file_name: PathBuf = GeneratedResources::write(
      &GeneratedResources::sample_name(file!(), "load_from_file.ltx"),
      b"[test]\nKey=Value\n",
    )
    .expect("generated LTX input to be written");

    let ltx: Ltx = Ltx::read_from_path(&file_name).unwrap();
    assert_eq!(ltx.get_from("test", "Key"), Some("Value"));
  }

  /// Format `input` through a file on disk and compare the result to `expected`, line by line.
  fn assert_formats(case: &str, input: &str, expected: &str) {
    let path: PathBuf =
      GeneratedResources::write(&GeneratedResources::sample_name(file!(), &format!("{case}.ltx")), input)
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

  /// The include statements a file holds, read through the document a project reads it as.
  fn includes_of(case: &str, contents: &[u8]) -> Vec<String> {
    let file_name: PathBuf = GeneratedResources::write(&GeneratedResources::sample_name(file!(), case), contents)
      .expect("generated LTX input to be written");

    Ltx::read_document_from(&mut std::fs::File::open(file_name).expect("generated LTX input to open"))
      .unwrap()
      .list_included()
      .into_iter()
      .map(String::from)
      .collect()
  }

  #[test]
  fn load_no_includes_from_file() {
    assert_eq!(
      includes_of("load_no_includes.ltx", b"[test]Key=Value\n"),
      Vec::<String>::new()
    );
  }

  #[test]
  fn load_few_includes_from_file() {
    assert_eq!(
      includes_of(
        "load_few_includes.ltx",
        b"#include \"first.ltx\"\n;commented\n#include \"second.ltx\""
      ),
      vec!("first.ltx", "second.ltx")
    );
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
