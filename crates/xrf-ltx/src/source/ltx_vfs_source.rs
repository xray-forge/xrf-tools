use std::sync::Arc;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{decode_bytes_to_string, new_windows1251_encoder};
use xrf_vfs::{XrayLogicalPath, XraySearch};

use crate::document::LtxDocument;
use crate::ltx::Ltx;
use crate::project::LtxReadCounters;
use crate::source::{LtxDocumentSource, LtxIncludeMask, LtxListingCache};

/// Resolves and reads includes through a search, so whatever records the search's reads records a config's too.
pub(crate) struct LtxVfsSource<'a> {
  search: XraySearch,
  /// Where this source reports its reads, when a project is counting them.
  counters: Option<&'a LtxReadCounters>,
  /// Where this source keeps directory listings, when a project shares them across the sources it creates.
  listings: Option<&'a LtxListingCache>,
}

impl<'a> LtxVfsSource<'a> {
  pub fn new(search: XraySearch) -> Self {
    Self {
      counters: None,
      listings: None,
      search,
    }
  }

  /// A source that reports every read and parse it performs.
  pub fn new_counted(search: XraySearch, counters: &'a LtxReadCounters) -> Self {
    Self {
      counters: Some(counters),
      listings: None,
      search,
    }
  }

  /// This source, answering directory listings from a cache shared with other sources over the same search.
  pub fn with_listings(mut self, listings: &'a LtxListingCache) -> Self {
    self.listings = Some(listings);
    self
  }

  /// Reads one logical path as a document, through whatever the VFS snapshot retains.
  pub fn read_document(&self, logical_path: &str) -> XrfResult<Arc<LtxDocument>> {
    let Some(asset) = self.search.find(logical_path)? else {
      // Absence is `NotFound`, as a VFS read answers it, so a caller can tell a missing config from a broken one.
      return Err(XrfError::new_not_found_error(format!(
        "no config '{logical_path}' in the searched sources"
      )));
    };

    self.search.read_parsed(&asset, |bytes| {
      if let Some(counters) = self.counters {
        counters.record_read(bytes.len() as u64);
        counters.record_parse();
      }

      Ltx::read_document_from_str(&decode_bytes_to_string(&bytes, new_windows1251_encoder())?)
    })
  }

  /// Reads only a config's include statements, without lowering its sections.
  pub fn read_included(&self, logical_path: &str) -> XrfResult<Vec<String>> {
    if let Some(counters) = self.counters {
      counters.record_include_scan();
    }

    Ok(
      self
        .read_document(logical_path)?
        .list_included()
        .into_iter()
        .map(String::from)
        .collect(),
    )
  }

  /// The sorted file names directly in `directory`, asked of the search every time.
  fn list_file_names_uncached(&self, directory: &str) -> XrfResult<Vec<String>> {
    let mut names: Vec<String> = self
      .search
      .list_children(directory)?
      .files
      .into_iter()
      .map(|location| String::from(location.get_logical_path().file_name()))
      .collect();

    names.sort();

    Ok(names)
  }
}

impl LtxDocumentSource for LtxVfsSource<'_> {
  fn read_document(&self, logical_path: &str) -> XrfResult<Option<Arc<LtxDocument>>> {
    if self.search.find(logical_path)?.is_none() {
      return Ok(None);
    }

    self.read_document(logical_path).map(Some)
  }

  /// Resolved as the engine reads a statement, against the logical directory of the config that wrote it, with either
  /// slash taken as its separator.
  fn resolve_include(&self, directory: &str, statement: &str) -> XrfResult<Vec<String>> {
    let directory: String = directory.replace('/', "\\");
    let statement: String = statement.replace('/', "\\");
    let joined: String = if directory.is_empty() {
      statement
    } else {
      format!("{directory}\\{statement}")
    };
    let normalized: String = XrayLogicalPath::normalize(&joined)?;

    if !LtxIncludeMask::is_mask(&normalized) {
      return Ok(vec![normalized]);
    }

    let (prefix, mask) = normalized.rsplit_once('\\').unwrap_or(("", &normalized));

    // `#include "sections\*.ltx"` means that one directory, so this asks for its children rather than everything below it.
    let mut resolved: Vec<String> = self
      .search
      .list_children(prefix)?
      .files
      .into_iter()
      .filter(|location| LtxIncludeMask::matches(location.get_logical_path().file_name().as_bytes(), mask.as_bytes()))
      .map(|location| String::from(location.get_logical_path().as_str()))
      .collect();

    // Sorted so section merging is deterministic, matching what the filesystem source guarantees.
    resolved.sort();

    Ok(resolved)
  }

  fn list_file_names(&self, directory: &str) -> XrfResult<Vec<String>> {
    match self.listings {
      Some(listings) => Ok(
        listings
          .get_or_list(directory, || self.list_file_names_uncached(directory))?
          .to_vec(),
      ),
      None => self.list_file_names_uncached(directory),
    }
  }
}
