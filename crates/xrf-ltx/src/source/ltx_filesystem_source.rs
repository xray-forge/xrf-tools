use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::document::LtxDocument;
use crate::ltx::{Ltx, LtxIncludeConvertor};
use crate::source::{LtxDocumentSource, LtxIncludeSource};

/// Resolves and reads includes from the filesystem, which is what an LTX file read by path uses.
#[derive(Default)]
pub struct LtxFilesystemSource;

impl LtxIncludeSource for LtxFilesystemSource {
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

impl LtxDocumentSource for LtxFilesystemSource {
  fn read_document(&self, logical_path: &str) -> XrfResult<Option<Arc<LtxDocument>>> {
    let path: PathBuf = PathBuf::from(logical_path);

    match std::fs::File::open(&path) {
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

  fn resolve_include(&self, directory: &str, statement: &str) -> XrfResult<Vec<String>> {
    Ok(
      LtxIncludeConvertor::resolve_include_paths(Path::new(directory), statement)?
        .into_iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect(),
    )
  }

  fn list_file_names(&self, directory: &str) -> XrfResult<Vec<String>> {
    let entries = match std::fs::read_dir(directory) {
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
  /// Whether a `.ts` counterpart of an absent `.ltx` exists, meaning the config is generated and not yet built.
  fn is_raw_ts_variant_existing(path: &Path) -> bool {
    if path.extension().is_some_and(|extension| extension == "ltx") {
      path.with_extension("ts").exists()
    } else {
      false
    }
  }
}
