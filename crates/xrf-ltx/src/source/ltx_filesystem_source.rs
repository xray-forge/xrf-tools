use std::ffi::{OsStr, OsString};
use std::path::{MAIN_SEPARATOR_STR, Path, PathBuf};
use std::sync::Arc;
use std::{fs, io};

use xrf_error::{XrfError, XrfResult};
use xrf_extension::XrayExtension;
use xrf_utils::format_path;

use crate::document::LtxDocument;
use crate::ltx::Ltx;
use crate::source::{LtxDocumentSource, LtxIncludeMask};

/// Resolves and reads configs on the host filesystem, which is what an LTX file read by path uses.
#[derive(Default)]
pub struct LtxFilesystemSource;

impl LtxDocumentSource for LtxFilesystemSource {
  fn read_document(&self, logical_path: &str) -> XrfResult<Option<Arc<LtxDocument>>> {
    let path: PathBuf = PathBuf::from(logical_path);

    match fs::File::open(&path) {
      Ok(mut file) => Ok(Some(Arc::new(Ltx::read_document_from(&mut file)?))),
      // A config generated from TypeScript is absent until the project is built, and a project that has not been built
      // must still parse.
      Err(error) if error.kind() == io::ErrorKind::NotFound && Self::is_raw_ts_variant_existing(&path) => Ok(None),
      Err(error) => Err(XrfError::new_io_error(
        format!("Failed to read included ltx file '{}': {error}", format_path(&path)),
        error.kind(),
      )),
    }
  }

  /// Refuses a match whose name is not valid Unicode, as the VFS refuses to index one: a source addresses configs by
  /// text, and a lossy name would read a different file or none, silently.
  fn resolve_include(&self, directory: &str, statement: &str) -> XrfResult<Vec<String>> {
    Self::resolve_include_paths(Path::new(directory), statement)?
      .into_iter()
      .map(|path| {
        path.into_os_string().into_string().map_err(|path| {
          XrfError::new_convert_error(format!(
            "Failed to include ltx file '{}', its name is not valid Unicode",
            format_path(Path::new(&path))
          ))
        })
      })
      .collect()
  }

  fn list_file_names(&self, directory: &str) -> XrfResult<Vec<String>> {
    let entries = match fs::read_dir(directory) {
      Ok(entries) => entries,
      Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
      Err(error) => return Err(error.into()),
    };

    let mut names: Vec<String> = entries
      .flatten()
      .filter(|entry| entry.path().is_file())
      .map(|entry| entry.file_name().to_string_lossy().into_owned())
      .collect();

    names.sort();

    Ok(names)
  }
}

impl LtxFilesystemSource {
  /// The files an include statement names in `directory`, sorted so section merging is deterministic.
  ///
  /// The statement's `\` separators are the engine's, so they are turned into the host's before joining. A mask such as
  /// `w_*.ltx` loads every matching file directly in its own directory, and every directory entry is weighed against it,
  /// including one whose name is not valid Unicode (see [`LtxIncludeMask::matches`]).
  fn resolve_include_paths(directory: &Path, statement: &str) -> XrfResult<Vec<PathBuf>> {
    let included_path: PathBuf = directory.join(statement.replace('\\', MAIN_SEPARATOR_STR));

    if !LtxIncludeMask::is_mask(statement) {
      return Ok(vec![included_path]);
    }

    let (Some(parent), Some(mask)) = (
      included_path.parent(),
      included_path.file_name().map(OsStr::as_encoded_bytes),
    ) else {
      return Err(XrfError::new_convert_error(format!(
        "Failed to resolve the directory of wildcard include {statement}"
      )));
    };

    let entries: fs::ReadDir = match fs::read_dir(parent) {
      Ok(entries) => entries,
      Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
      Err(error) => return Err(error.into()),
    };
    let mut resolved: Vec<PathBuf> = Vec::new();

    for entry in entries {
      let entry: fs::DirEntry = entry?;
      let file_name: OsString = entry.file_name();

      if LtxIncludeMask::matches(file_name.as_encoded_bytes(), mask) && entry.path().is_file() {
        resolved.push(entry.path());
      }
    }

    resolved.sort();

    Ok(resolved)
  }

  /// Whether a `.ts` counterpart of an absent `.ltx` exists, meaning the config is generated and not yet built.
  fn is_raw_ts_variant_existing(path: &Path) -> bool {
    XrayExtension::Ltx.matches_path(path) && path.with_extension(XrayExtension::Ts.as_str()).exists()
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;

  use xrf_error::XrfResult;
  use xrf_test_utils::GeneratedResources;

  use crate::Ltx;

  #[test]
  fn loads_each_file_matched_by_wildcard_include() -> XrfResult {
    // The include is a wildcard, so a section file an earlier run left behind would join this run's matches.
    let root: PathBuf = GeneratedResources::create_dir("include/wildcard")?;
    let sections: PathBuf = root.join("sections");
    let root_ltx: PathBuf = root.join("root.ltx");

    fs::create_dir_all(&sections)?;
    fs::write(&root_ltx, "#include \"sections\\section_*.ltx\"\n")?;
    fs::write(sections.join("section_first.ltx"), "[first]\n")?;
    fs::write(sections.join("section_second.ltx"), "[second]\n")?;
    fs::write(sections.join("ignored.ltx"), "[ignored]\n")?;

    let ltx: Ltx = Ltx::read_from_file_standard(&root_ltx)?;

    assert!(ltx.has_section("first"));
    assert!(ltx.has_section("second"));
    assert!(!ltx.has_section("ignored"));

    fs::remove_dir_all(root)?;

    Ok(())
  }

  /// Linux filenames are bytes, not text, so a wildcard can match a file whose name is not valid Unicode. Every source
  /// addresses configs by text, so such a match is refused by name rather than skipped, which would leave its sections
  /// silently absent from the merged result.
  #[test]
  #[cfg(target_os = "linux")]
  fn refuses_a_wildcard_match_whose_file_name_is_not_valid_unicode() -> XrfResult {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let root: PathBuf = GeneratedResources::create_dir("include/wildcard_non_utf8")?;
    let sections: PathBuf = root.join("sections");
    let root_ltx: PathBuf = root.join("root.ltx");
    let non_unicode_name: &OsStr = OsStr::from_bytes(b"section_\xffbroken.ltx");

    assert!(non_unicode_name.to_str().is_none());

    fs::create_dir_all(&sections)?;
    fs::write(&root_ltx, "#include \"sections\\section_*.ltx\"\n")?;
    fs::write(sections.join("section_readable.ltx"), "[readable]\n")?;
    fs::write(sections.join(non_unicode_name), "[non_unicode]\n")?;

    let error: String = Ltx::read_from_file_standard(&root_ltx)
      .expect_err("a match whose name is not valid Unicode to be refused")
      .to_string();

    assert!(error.contains("not valid Unicode"), "{error}");
    assert!(error.contains("section_"), "{error}");

    // Not left behind: the CI cache archives `target/` with tar, which cannot read a name that is not valid Unicode.
    fs::remove_dir_all(root)?;

    Ok(())
  }
}
