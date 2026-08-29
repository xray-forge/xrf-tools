use std::io;
use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::Ltx;
use crate::file::include::LtxIncludeConvertor;
use crate::file::include_source::LtxIncludeSource;

/// Resolves and reads includes from the filesystem, which is what an LTX file read by path uses.
#[derive(Default)]
pub(crate) struct LtxIncludeFilesystemSource;

impl LtxIncludeSource for LtxIncludeFilesystemSource {
  fn resolve(&self, directory: &Path, statement: &str) -> XrfResult<Vec<PathBuf>> {
    LtxIncludeConvertor::resolve_include_paths(directory, statement)
  }

  /// Reads a nested file, treating an absent one with a `.ts` counterpart as nothing to include.
  ///
  /// A config generated from TypeScript is absent until the project is built, and a project that has not been built yet must
  /// still parse.
  fn read(&self, path: &Path) -> XrfResult<Option<Ltx>> {
    match Ltx::read_from_path(path) {
      Ok(ltx) => Ok(Some(ltx)),
      Err(error) => match error {
        XrfError::Io { ref kind, message: _ } => {
          if *kind == io::ErrorKind::NotFound && Self::is_raw_ts_variant_existing(path) {
            Ok(None)
          } else {
            Err(error)
          }
        }
        _ => Err(error),
      },
    }
  }

  fn describe(&self, path: &Path) -> String {
    format_path(path).to_string()
  }
}

impl LtxIncludeFilesystemSource {
  /// Whether a `.ts` counterpart of an absent `.ltx` exists, meaning the config is generated and not yet built.
  fn is_raw_ts_variant_existing(path: &Path) -> bool {
    if path.extension().is_some_and(|extension| extension == "ltx") {
      path.with_extension("ts").exists()
    } else {
      false
    }
  }
}
