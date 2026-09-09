use std::cell::Cell;
use std::collections::BTreeMap;
use std::sync::Arc;

use xrf_error::XrfResult;
use xrf_ltx::{
  LTX_SYMBOL_INCLUDE_WILDCARD, Ltx, LtxDialect, LtxDocument, LtxDocumentSource, LtxResolution, LtxResolveRequest,
  LtxStandardDialect,
};

/// A [`LtxDocumentSource`] over configs held in memory.
///
/// Contents are kept as text and parsed on every read rather than once on the way in, so a config that will not parse
/// is a thing the source answers with rather than a thing that cannot be built - which is the case the structure
/// record and the parse finding both exist for.
#[derive(Debug, Default)]
pub struct LtxMapSource {
  contents: BTreeMap<String, String>,
  /// How many documents have been read out of this source.
  ///
  /// A seam rather than an accessory: whether a reader reads a declaring config once or once per question is a
  /// property nothing else can observe, and it is the difference between a page turn costing a lookup and costing a
  /// tree of parses.
  reads: Cell<usize>,
}

impl LtxMapSource {
  /// Builds a source from `(logical path, contents)` pairs.
  ///
  /// Paths are lowercased on the way in, which is what the engine's own file table does to every name it registers.
  pub fn new(files: &[(&str, &str)]) -> Self {
    Self {
      contents: files
        .iter()
        .map(|(path, contents)| (path.to_lowercase(), String::from(*contents)))
        .collect(),
      reads: Cell::new(0),
    }
  }

  /// Resolves one root under standard LTX, recording where every field came from.
  pub fn resolve(&self, root: &str) -> XrfResult<LtxResolution> {
    LtxStandardDialect.resolve(root, self, LtxResolveRequest::with_provenance())
  }

  /// How many documents this source has been asked for so far.
  pub fn reads(&self) -> usize {
    self.reads.get()
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
    match mask.split_once(LTX_SYMBOL_INCLUDE_WILDCARD) {
      Some((prefix, suffix)) => {
        name.len() >= prefix.len() + suffix.len() && name.starts_with(prefix) && name.ends_with(suffix)
      }
      None => name == mask,
    }
  }
}

impl LtxDocumentSource for LtxMapSource {
  fn read_document(&self, logical_path: &str) -> XrfResult<Option<Arc<LtxDocument>>> {
    self.reads.set(self.reads.get() + 1);

    match self.contents.get(&logical_path.to_lowercase()) {
      Some(contents) => Ok(Some(Arc::new(Ltx::read_document_from_str(contents)?))),
      None => Ok(None),
    }
  }

  fn resolve_include(&self, directory: &str, statement: &str) -> XrfResult<Vec<String>> {
    let joined: String = if directory.is_empty() {
      String::from(statement)
    } else {
      format!("{directory}\\{statement}")
    }
    .to_lowercase();

    if !statement.contains(LTX_SYMBOL_INCLUDE_WILDCARD) {
      return Ok(vec![joined]);
    }

    let mask_directory: &str = Self::directory_of(&joined);
    let mask: &str = Self::file_name_of(&joined);

    // Sorted, because the engine's file table is ordered and merge order must not depend on iteration order.
    let mut matched: Vec<String> = self
      .contents
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
      .contents
      .keys()
      .filter(|path| Self::directory_of(path) == directory)
      .map(|path| String::from(Self::file_name_of(path)))
      .collect();

    names.sort();

    Ok(names)
  }
}
