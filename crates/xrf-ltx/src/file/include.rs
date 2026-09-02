use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::{MAIN_SEPARATOR_STR, Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path_or;

use crate::Ltx;
use crate::file::file_configuration::constants::VIRTUAL_LTX_PATH;
use crate::file::include_filesystem_source::LtxIncludeFilesystemSource;
use crate::file::include_source::LtxIncludeSource;

/// Converter object to process and inject all child #include statements.
#[derive(Default)]
pub struct LtxIncludeConvertor {}

impl LtxIncludeConvertor {
  /// Cast LTX file to fully parsed with include sections, reading them from the filesystem.
  pub fn convert(ltx: Ltx) -> XrfResult<Ltx> {
    Self::convert_with(ltx, &LtxIncludeFilesystemSource)
  }

  /// Cast LTX file to fully parsed, reading includes from a given source.
  ///
  /// The merge rules live here and nowhere else: which file a statement names differs between backends, what merging two
  /// sections means does not.
  pub(crate) fn convert_with<S: LtxIncludeSource>(ltx: Ltx, source: &S) -> XrfResult<Ltx> {
    Self {}.convert_ltx(ltx, source)
  }

  /// Transform ltx statement to cross-platform path.
  pub fn statement_to_path(statement: &str) -> PathBuf {
    PathBuf::from(statement.replace('\\', MAIN_SEPARATOR_STR))
  }

  /// Resolve an include statement to files in its containing directory.
  ///
  /// X-Ray extensions accepts `*` masks such as `w_*.ltx` and loads every matching file
  /// directly from the include directory. Matches are sorted so that section
  /// merging is deterministic across filesystems.
  ///
  /// Every directory entry is compared, including one whose name is not valid Unicode: a Unix filename is bytes, and such a
  /// file is a config the engine would load. The comparison is therefore over encoded bytes rather than `str`, so a name that
  /// does not decode still matches a mask whose literal parts it carries.
  pub fn resolve_include_paths<P: AsRef<Path>>(directory: P, statement: &str) -> XrfResult<Vec<PathBuf>> {
    let included_path: PathBuf = directory.as_ref().join(Self::statement_to_path(statement));

    if !statement.contains('*') {
      return Ok(vec![included_path]);
    }

    let Some(parent) = included_path.parent() else {
      return Err(XrfError::new_convert_error(format!(
        "Failed to resolve parent directory for wildcard include {statement}"
      )));
    };

    let Some(mask) = included_path.file_name().map(OsStr::as_encoded_bytes) else {
      return Err(XrfError::new_convert_error(format!(
        "Failed to resolve wildcard file name for include {statement}"
      )));
    };

    let entries: fs::ReadDir = match fs::read_dir(parent) {
      Ok(entries) => entries,
      Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
      Err(error) => return Err(error.into()),
    };
    let mut resolved_paths: Vec<PathBuf> = Vec::new();

    for entry in entries {
      let entry: fs::DirEntry = entry?;
      let file_name: OsString = entry.file_name();

      if !Self::matches_wildcard_mask(file_name.as_encoded_bytes(), mask) {
        continue;
      }

      let path: PathBuf = entry.path();

      if path.is_file() {
        resolved_paths.push(path);
      }
    }

    resolved_paths.sort();

    Ok(resolved_paths)
  }
}

impl LtxIncludeConvertor {
  /// Convert ltx file with inclusion of nested files.
  fn convert_ltx<S: LtxIncludeSource>(&self, ltx: Ltx, source: &S) -> XrfResult<Ltx> {
    if ltx.directory.is_none() {
      return Err(XrfError::new_convert_error(
        "Failed to parse ltx file, parent directory is not specified",
      ));
    }

    // Nothing to parse - no include statements.
    if ltx.includes.is_empty() {
      return Ok(ltx);
    }

    let mut result: Ltx = Ltx {
      path: ltx.path,
      directory: ltx.directory,
      includes: Default::default(),
      skipped_checks: ltx.skipped_checks,
      sections: Default::default(),
    };

    for included in &ltx.includes {
      let included_paths: Vec<PathBuf> = source.resolve(result.directory.as_ref().unwrap(), included)?;

      for included_path in included_paths {
        self.include_children(&mut result, &included_path, source)?;
      }
    }

    for (key, value) in ltx.sections {
      match result.section_mut(&key) {
        None => {
          result.sections.insert(key, value);
        }
        Some(existing) => {
          // Handle cases with root declarations.
          if key.is_empty() {
            existing.merge(value);
          } else {
            return Err(XrfError::new_convert_error(format!(
              "Failed to equipment ltx file, duplicate section {key} found",
            )));
          }
        }
      }
    }

    Ok(result)
  }

  /// Include children ltx into provided ltx.
  fn include_children<S: LtxIncludeSource>(&self, into: &mut Ltx, path: &Path, source: &S) -> XrfResult {
    let ltx: Ltx = match source.read(path) {
      Ok(value) => match value {
        Some(ltx) => ltx,
        None => return Ok(()),
      },
      Err(error) => {
        return Err(XrfError::new_convert_error(format!(
          "Failed to parse ltx file, nested file {} in {} error: {error}",
          source.describe(path),
          format_path_or(into.path.as_deref(), VIRTUAL_LTX_PATH),
        )));
      }
    };

    for (key, value) in Self::convert_with(ltx, source)?.sections {
      match into.section_mut(&key) {
        None => {
          into.sections.insert(key, value);
        }
        Some(existing) => {
          // Handle cases with root declarations.
          if key.is_empty() {
            existing.merge(value);
          } else {
            return Err(XrfError::new_convert_error(format!(
              "Failed to include ltx file '{}' in {}, duplicate section '{}' found",
              source.describe(path),
              format_path_or(into.path.as_deref(), VIRTUAL_LTX_PATH),
              key
            )));
          }
        }
      }
    }

    Ok(())
  }

  /// Open nested file for importing in current context.
  /// Skips '.ts' variant of configuration file as None.
  fn parse_nested_file<P: AsRef<Path>>(&self, path: &P) -> XrfResult<Option<Ltx>> {
    match Ltx::read_from_path(path.as_ref()) {
      Ok(ltx) => Ok(Some(ltx)),
      Err(error) => match error {
        XrfError::Io { ref kind, message: _ } => {
          if *kind == io::ErrorKind::NotFound {
            if self.is_raw_ts_variant_existing(path) {
              Ok(None)
            } else {
              Err(error)
            }
          } else {
            Err(error)
          }
        }
        _ => Err(error),
      },
    }
  }

  /// Check if similar TS counterpart exists for provided ltx path.
  fn is_raw_ts_variant_existing<P: AsRef<Path>>(&self, path: &P) -> bool {
    if path.as_ref().extension().is_some_and(|extension| extension == "ltx") {
      path.as_ref().with_extension("ts").exists()
    } else {
      false
    }
  }

  /// Whether a file name matches a `*` mask, both as encoded bytes.
  ///
  /// Bytes rather than `str` because a filename is not required to be valid Unicode. Both encodings are self-synchronizing
  /// supersets of UTF-8, so searching a mask part cannot match inside a multi-byte sequence.
  pub(crate) fn matches_wildcard_mask(file_name: &[u8], mask: &[u8]) -> bool {
    let mut remaining: &[u8] = file_name;
    let mut is_first_part: bool = true;

    for part in mask.split(|byte| *byte == b'*').filter(|part| !part.is_empty()) {
      let Some(position) = Self::find_subslice(remaining, part) else {
        return false;
      };

      if is_first_part && !mask.starts_with(b"*") && position != 0 {
        return false;
      }

      remaining = &remaining[position + part.len()..];
      is_first_part = false;
    }

    mask.ends_with(b"*") || remaining.is_empty()
  }

  /// Offset of the first occurrence of `needle`, which an empty one finds at the start.
  fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
      return Some(0);
    }

    if needle.len() > haystack.len() {
      return None;
    }

    haystack.windows(needle.len()).position(|window| window == needle)
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;

  use xrf_error::XrfResult;

  use crate::Ltx;
  use crate::file::include::LtxIncludeConvertor;

  #[test]
  fn loads_each_file_matched_by_wildcard_include() -> XrfResult {
    let root: PathBuf = std::env::temp_dir().join(format!("xrf-ltx-wildcard-include-{}", std::process::id()));
    let sections: PathBuf = root.join("sections");
    let root_ltx: PathBuf = root.join("root.ltx");

    fs::create_dir_all(&sections)?;
    fs::write(&root_ltx, "#include \"sections\\section_*.ltx\"\n")?;
    fs::write(sections.join("section_first.ltx"), "[first]\n")?;
    fs::write(sections.join("section_second.ltx"), "[second]\n")?;
    fs::write(sections.join("ignored.ltx"), "[ignored]\n")?;

    let ltx: Ltx = Ltx::read_from_file_included(&root_ltx)?;

    assert!(ltx.has_section("first"));
    assert!(ltx.has_section("second"));
    assert!(!ltx.has_section("ignored"));

    fs::remove_dir_all(root)?;

    Ok(())
  }

  /// Linux filenames are bytes, not text, so a config the engine loads can carry a name that is not valid Unicode. The
  /// wildcard resolver used to convert every directory entry with `to_str` and skip the ones that answered `None`, leaving
  /// the sections such a file declares silently absent from the merged result.
  #[test]
  #[cfg(target_os = "linux")]
  fn loads_a_wildcard_match_whose_file_name_is_not_valid_unicode() -> XrfResult {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let root: PathBuf = std::env::temp_dir().join(format!("xrf-ltx-wildcard-non-utf8-{}", std::process::id()));
    let sections: PathBuf = root.join("sections");
    let root_ltx: PathBuf = root.join("root.ltx");
    let non_unicode_name: &OsStr = OsStr::from_bytes(b"section_\xffbroken.ltx");

    assert!(non_unicode_name.to_str().is_none());

    fs::create_dir_all(&sections)?;
    fs::write(&root_ltx, "#include \"sections\\section_*.ltx\"\n")?;
    fs::write(sections.join("section_readable.ltx"), "[readable]\n")?;
    fs::write(sections.join(non_unicode_name), "[non_unicode]\n")?;

    let ltx: Ltx = Ltx::read_from_file_included(&root_ltx)?;

    // The readable sibling guards against a regression that drops both instead of neither.
    assert!(ltx.has_section("readable"));
    assert!(ltx.has_section("non_unicode"));

    fs::remove_dir_all(root)?;

    Ok(())
  }

  /// The mask comparison is over bytes, so a name that does not decode is still weighed against the mask instead of being
  /// dropped. This runs everywhere; the filesystem end of it is Unix-only because Windows cannot hold such a name.
  #[test]
  fn matches_a_wildcard_mask_over_bytes_that_are_not_valid_unicode() {
    // 0xFF cannot appear in UTF-8, so this name is only reachable as bytes.
    let name: &[u8] = b"section_\xffbroken.ltx";

    assert!(LtxIncludeConvertor::matches_wildcard_mask(name, b"section_*.ltx"));
    assert!(LtxIncludeConvertor::matches_wildcard_mask(name, b"*broken*"));
    assert!(!LtxIncludeConvertor::matches_wildcard_mask(name, b"weapon_*.ltx"));
    assert!(!LtxIncludeConvertor::matches_wildcard_mask(name, b"section_*.xml"));
  }
}
