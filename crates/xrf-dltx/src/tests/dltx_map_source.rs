use std::collections::BTreeMap;
use std::sync::Arc;

use xrf_error::XrfResult;
use xrf_ltx::{Ltx, LtxDocument, LtxDocumentSource};

/// A [`LtxDocumentSource`] over configs held in memory.
///
/// What lets every rule in the compatibility matrix be tested without a filesystem: a row is a handful of config
/// strings and one expected resolution. Also the shape an integration reads against, so a caller can check its own
/// wiring against the same behaviour.
#[derive(Debug, Default)]
pub struct DltxMapSource {
  documents: BTreeMap<String, Arc<LtxDocument>>,
}

impl DltxMapSource {
  /// Builds a source from `(logical path, contents)` pairs.
  ///
  /// Paths are lowercased on the way in, which is what the engine's own file table does to every name it registers.
  ///
  /// # Errors
  ///
  /// Returns an error when any of the contents will not parse.
  pub fn new(files: &[(&str, &str)]) -> XrfResult<Self> {
    let mut documents: BTreeMap<String, Arc<LtxDocument>> = BTreeMap::new();

    for (path, contents) in files {
      documents.insert(path.to_lowercase(), Arc::new(Ltx::read_document_from_str(contents)?));
    }

    Ok(Self { documents })
  }

  /// Everything before the last separator, or the empty string for a top-level name.
  fn directory_of(logical_path: &str) -> &str {
    match logical_path.rsplit_once('\\') {
      Some((directory, _)) => directory,
      None => "",
    }
  }

  /// The last segment of a logical path.
  fn file_name_of(logical_path: &str) -> &str {
    match logical_path.rsplit_once('\\') {
      Some((_, name)) => name,
      None => logical_path,
    }
  }

  /// Whether `name` matches a mask that may carry one `*`.
  fn matches(name: &str, mask: &str) -> bool {
    match mask.split_once('*') {
      Some((prefix, suffix)) => {
        name.len() >= prefix.len() + suffix.len() && name.starts_with(prefix) && name.ends_with(suffix)
      }
      None => name == mask,
    }
  }
}

impl LtxDocumentSource for DltxMapSource {
  fn read_document(&self, logical_path: &str) -> XrfResult<Option<Arc<LtxDocument>>> {
    Ok(self.documents.get(&logical_path.to_lowercase()).cloned())
  }

  fn resolve_include(&self, directory: &str, statement: &str) -> XrfResult<Vec<String>> {
    let joined: String = if directory.is_empty() {
      String::from(statement)
    } else {
      format!("{directory}\\{statement}")
    }
    .to_lowercase();

    if !statement.contains('*') {
      return Ok(vec![joined]);
    }

    let mask_directory: &str = Self::directory_of(&joined);
    let mask: &str = Self::file_name_of(&joined);

    // Sorted, because the engine's file table is ordered and merge order must not depend on iteration order.
    let mut matched: Vec<String> = self
      .documents
      .keys()
      .filter(|path| Self::directory_of(path) == mask_directory)
      .filter(|path| Self::matches(Self::file_name_of(path), mask))
      .cloned()
      .collect();

    matched.sort();

    Ok(matched)
  }

  fn list_file_names(&self, directory: &str) -> XrfResult<Vec<String>> {
    let directory: String = directory.to_lowercase();

    let mut names: Vec<String> = self
      .documents
      .keys()
      .filter(|path| Self::directory_of(path) == directory)
      .map(|path| String::from(Self::file_name_of(path)))
      .collect();

    names.sort();

    Ok(names)
  }
}
