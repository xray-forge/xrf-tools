use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_utils::{
  decode_bytes_to_string, encode_w1251_bytes_to_string, new_windows1251_encoder, read_as_string_from_w1251_encoded,
};
use xrf_vfs::{XrayLookupScope, XrayVfs};

use crate::Ltx;
use crate::file::include::LtxIncludeConvertor;
use crate::file::include_vfs_source::LtxIncludeVfsSource;
use crate::file::parser::LtxParser;
use crate::file::types::LtxIncluded;

impl Ltx {
  /// Read LTX from a string.
  pub fn read_from_str(buf: &str) -> XrfResult<Self> {
    LtxParser::new(buf.chars()).parse()
  }

  /// Read LTX from a file as full parsed file, inject included files.
  pub fn read_from_file_included<P: AsRef<Path>>(filename: P) -> XrfResult<Self> {
    Self::read_from_path(filename)?.into_included()
  }

  /// Read LTX from a file, inject all includes and unwrap inherited sections.
  pub fn read_from_file_full<P: AsRef<Path>>(filename: P) -> XrfResult<Self> {
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
  pub fn read_from_vfs(vfs: &XrayVfs, scope: &XrayLookupScope, logical_path: &str) -> XrfResult<Self> {
    let source: LtxIncludeVfsSource = LtxIncludeVfsSource::new(vfs, scope);

    LtxIncludeConvertor::convert_with(source.read_ltx(logical_path)?, &source)
  }

  /// Read LTX out of a mounted VFS with includes injected and inherited sections unwrapped.
  ///
  /// # Errors
  ///
  /// Returns an error when reading fails, or when an inherited section cannot be resolved.
  pub fn read_from_vfs_full(vfs: &XrayVfs, scope: &XrayLookupScope, logical_path: &str) -> XrfResult<Self> {
    Self::read_from_vfs(vfs, scope, logical_path)?.into_inherited()
  }

  /// Read from a file as generic ltx with LTX descriptor filled.
  pub fn read_from_path<P: AsRef<Path>>(filename: P) -> XrfResult<Self> {
    let mut ltx: Self = Self::read_from(&mut File::open(filename.as_ref())?)?;

    ltx.path = Some(PathBuf::from(filename.as_ref()));
    ltx.directory = filename.as_ref().parent().map(PathBuf::from);

    Ok(ltx)
  }

  /// Read from a reader as generic ltx with LTX descriptor filled.
  pub fn read_from<R: Read>(reader: &mut R) -> XrfResult<Self> {
    LtxParser::new(read_as_string_from_w1251_encoded(reader)?.chars()).parse()
  }
}

impl Ltx {
  /// Load include statements from a string.
  pub fn read_included_from_str(buf: &str) -> XrfResult<LtxIncluded> {
    LtxParser::new(buf.chars()).parse_includes()
  }

  /// Load include statements from a file with options.
  pub fn read_included_from_file<P: AsRef<Path>>(filename: P) -> XrfResult<LtxIncluded> {
    Self::read_included_from(&mut File::open(filename.as_ref())?)
  }

  /// Load include statements from a config in a mounted VFS, without parsing its sections.
  ///
  /// Used when assembling a project: entry points are the files nothing else includes, which needs every file's include list
  /// and none of their contents.
  ///
  /// # Errors
  ///
  /// Returns an error when the path is not in scope, its bytes are not valid Windows-1251, or the contents will not parse.
  pub fn read_included_from_vfs(vfs: &XrayVfs, scope: &XrayLookupScope, logical_path: &str) -> XrfResult<LtxIncluded> {
    let bytes: Vec<u8> = vfs.scoped(scope).read_bytes(logical_path)?;

    Self::read_included_from_str(&decode_bytes_to_string(&bytes, new_windows1251_encoder())?)
  }

  /// Load include statements from a reader.
  pub fn read_included_from<R: Read>(reader: &mut R) -> XrfResult<LtxIncluded> {
    LtxParser::new(read_as_string_from_w1251_encoded(reader)?.chars()).parse_includes()
  }
}

impl Ltx {
  /// Load formatted LTX as string from string.
  pub fn format_from_str(buf: &str) -> XrfResult<String> {
    LtxParser::new(buf.chars()).parse_into_formatted()
  }

  /// Load formatted LTX as string from file.
  pub fn format_from_file<P: AsRef<Path>>(filename: P) -> XrfResult<String> {
    Self::format_from(&mut File::open(filename.as_ref())?)
  }

  /// Load formatted LTX as string from reader.
  pub fn format_from<R: Read>(reader: &mut R) -> XrfResult<String> {
    LtxParser::new(read_as_string_from_w1251_encoded(reader)?.chars()).parse_into_formatted()
  }

  /// Whether the given LTX bytes are already formatted canonically.
  ///
  /// Content is all a formatting verdict needs, so this answers for an archived config too — only rewriting one needs a
  /// file, which is why [`Self::format_file`] stays separate.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes cannot be decoded or parsed.
  pub fn is_formatted(bytes: &[u8]) -> XrfResult<bool> {
    let existing: String = encode_w1251_bytes_to_string(bytes)?;

    Ok(existing == Self::format_from_str(&existing)?)
  }
}

#[cfg(test)]
mod test {
  use std::env::temp_dir;
  use std::fs::File;
  use std::io::Write;
  use std::path::PathBuf;

  use xrf_test_utils::file::read_file_as_normalized_win_endl_string;
  use xrf_test_utils::utils::{build_absolute_test_file_path, build_absolute_test_resource_as_file};

  use crate::Ltx;
  use crate::file::types::LtxIncluded;

  #[test]
  fn load_from_file() {
    let file_name = temp_dir().join("rust_ini_load_from_file");
    let file_content = b"[test]\nKey=Value\n";

    {
      let mut file: File = File::create(&file_name).expect("create");
      file.write_all(file_content).expect("write");
    }

    let ltx: Ltx = Ltx::read_from_path(&file_name).unwrap();
    assert_eq!(ltx.get_from("test", "Key"), Some("Value"));
  }

  #[test]
  fn format_from_file_one() {
    let formatted: String =
      Ltx::format_from_file(build_absolute_test_file_path(file!(), "not_formatted_1.ltx")).unwrap();

    let expected: String = read_file_as_normalized_win_endl_string(
      &mut build_absolute_test_resource_as_file(file!(), "formatted_1.ltx").unwrap(),
    )
    .unwrap();

    assert_eq!(formatted, expected);
  }

  #[test]
  fn format_from_file_two() {
    let formatted: String =
      Ltx::format_from_file(build_absolute_test_file_path(file!(), "not_formatted_2.ltx")).unwrap();

    let expected: String = read_file_as_normalized_win_endl_string(
      &mut build_absolute_test_resource_as_file(file!(), "formatted_2.ltx").unwrap(),
    )
    .unwrap();

    assert_eq!(formatted, expected);
  }

  #[test]
  fn format_from_file_three() {
    let formatted: String =
      Ltx::format_from_file(build_absolute_test_file_path(file!(), "not_formatted_3.ltx")).unwrap();

    let expected: String = read_file_as_normalized_win_endl_string(
      &mut build_absolute_test_resource_as_file(file!(), "formatted_3.ltx").unwrap(),
    )
    .unwrap();

    assert_eq!(formatted, expected);
  }

  #[test]
  fn format_from_file_four() {
    let formatted: String =
      Ltx::format_from_file(build_absolute_test_file_path(file!(), "not_formatted_4.ltx")).unwrap();

    let expected: String = read_file_as_normalized_win_endl_string(
      &mut build_absolute_test_resource_as_file(file!(), "formatted_4.ltx").unwrap(),
    )
    .unwrap();

    assert_eq!(formatted, expected);
  }

  #[test]
  fn load_no_includes_from_file() {
    let file_name: PathBuf = temp_dir().join("rust_ini_load_no_includes");
    let file_content = b"[test]Key=Value\n";

    {
      let mut file: File = File::create(&file_name).expect("create");
      file.write_all(file_content).expect("write");
    }

    let includes: LtxIncluded = Ltx::read_included_from_file(&file_name).unwrap();
    assert_eq!(includes, Vec::<String>::new());
  }

  #[test]
  fn load_few_includes_from_file() {
    let file_name: PathBuf = temp_dir().join("rust_ini_load_from_file_without_bom");
    let file_content = b"#include \"first.ltx\"\n;commented\n#include \"second.ltx\"";

    {
      let mut file: File = File::create(&file_name).expect("create");
      file.write_all(file_content).expect("write");
    }

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
