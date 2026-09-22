use std::path::{Path, PathBuf};
use std::sync::Arc;

use xrf_error::XrfResult;
use xrf_utils::{decode_bytes_to_string, new_windows1251_encoder};
use xrf_vfs::{XrayAssetType, XrayLogicalPath, XrayLookupScope, XrayVfs};

use crate::dialect::LtxStandardDialect;
use crate::document::LtxDocument;
use crate::ltx::{Ltx, LtxIncludeConvertor};
use crate::project::LtxReadCounters;
use crate::source::{LtxDocumentSource, LtxIncludeSource, LtxListingCache};

/// Resolves and reads includes through a mounted VFS.
pub(crate) struct LtxVfsSource<'a> {
  vfs: &'a XrayVfs,
  scope: &'a XrayLookupScope,
  /// Where this source reports its reads, when a project is counting them.
  counters: Option<&'a LtxReadCounters>,
  /// Where this source keeps directory listings, when a project shares them across the sources it creates.
  listings: Option<&'a LtxListingCache>,
}

impl<'a> LtxVfsSource<'a> {
  pub fn new(vfs: &'a XrayVfs, scope: &'a XrayLookupScope) -> Self {
    Self {
      counters: None,
      listings: None,
      scope,
      vfs,
    }
  }

  /// A source that reports every read and parse it performs.
  pub fn new_counted(vfs: &'a XrayVfs, scope: &'a XrayLookupScope, counters: &'a LtxReadCounters) -> Self {
    Self {
      counters: Some(counters),
      listings: None,
      scope,
      vfs,
    }
  }

  /// This source, answering directory listings from a cache shared with other sources over the same VFS and scope.
  pub fn with_listings(mut self, listings: &'a LtxListingCache) -> Self {
    self.listings = Some(listings);
    self
  }

  /// Reads one logical path as a document, through whatever the mounted world retains.
  pub fn read_document(&self, logical_path: &str) -> XrfResult<Arc<LtxDocument>> {
    self
      .vfs
      .scoped(self.scope)
      .read_parsed(XrayAssetType::Ltx, logical_path, |bytes| {
        if let Some(counters) = self.counters {
          counters.record_read(bytes.len() as u64);
          counters.record_parse();
        }

        Ltx::read_document_from_str(&decode_bytes_to_string(&bytes, new_windows1251_encoder())?)
      })
  }

  /// Reads and lowers one logical path, with its logical location recorded so nested includes resolve against it.
  pub fn read_ltx(&self, logical_path: &str) -> XrfResult<Ltx> {
    let mut ltx: Ltx = LtxStandardDialect::lower(self.read_document(logical_path)?.as_ref())?;
    let path: XrayLogicalPath = XrayLogicalPath::new(logical_path)?;

    // The logical parent, not `Path::parent`: on a host that does not separate on `\` the latter answers the path unsplit, and
    // every nested include then resolves against the mount root. An empty directory stands for a top-level config, which is
    // what a filesystem read records for one too.
    ltx.directory = Some(PathBuf::from(
      path
        .parent()
        .map_or_else(String::new, |parent| parent.as_str().to_string()),
    ));
    ltx.path = Some(PathBuf::from(path.as_str()));

    Ok(ltx)
  }

  /// Reads only a config's include statements, without parsing its sections.
  pub fn read_included(&self, logical_path: &str) -> XrfResult<crate::ltx::LtxIncluded> {
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

  /// A logical path as the VFS spells it.
  fn to_logical(path: &Path) -> String {
    path.to_string_lossy().replace('/', "\\")
  }

  /// The sorted file names directly in `directory`, asked of the VFS every time.
  fn list_file_names_uncached(&self, directory: &str) -> XrfResult<Vec<String>> {
    let mut names: Vec<String> = self
      .vfs
      .scoped(self.scope)
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
    if self.vfs.scoped(self.scope).find(logical_path)?.is_none() {
      return Ok(None);
    }

    self.read_document(logical_path).map(Some)
  }

  fn resolve_include(&self, directory: &str, statement: &str) -> XrfResult<Vec<String>> {
    Ok(
      LtxIncludeSource::resolve(self, &PathBuf::from(directory), statement)?
        .into_iter()
        .map(|path| Self::to_logical(&path))
        .collect(),
    )
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

impl LtxIncludeSource for LtxVfsSource<'_> {
  fn resolve(&self, directory: &Path, statement: &str) -> XrfResult<Vec<PathBuf>> {
    let directory: String = Self::to_logical(directory);
    let statement: String = statement.replace('/', "\\");

    let joined: String = if directory.is_empty() {
      statement.clone()
    } else {
      format!("{directory}\\{statement}")
    };

    if !statement.contains('*') {
      return Ok(vec![PathBuf::from(XrayLogicalPath::normalize(&joined)?)]);
    }

    let normalized: String = XrayLogicalPath::normalize(&joined)?;
    let (prefix, mask) = match normalized.rsplit_once('\\') {
      Some((prefix, mask)) => (prefix.to_string(), mask.to_string()),
      None => (String::new(), normalized.clone()),
    };

    // `#include "sections\*.ltx"` means that one directory, so this asks for its children rather than everything below it.
    let mut resolved: Vec<PathBuf> = self
      .vfs
      .scoped(self.scope)
      .list_children(&prefix)?
      .files
      .into_iter()
      .filter(|location| {
        LtxIncludeConvertor::matches_wildcard_mask(location.get_logical_path().file_name().as_bytes(), mask.as_bytes())
      })
      // The documented crossing back into `PathBuf`: an include source carries logical paths that way for both backends.
      .map(|location| PathBuf::from(location.get_logical_path().as_str()))
      .collect();

    // Sorted so section merging is deterministic, matching what the filesystem source guarantees.
    resolved.sort();

    Ok(resolved)
  }

  fn read(&self, path: &Path) -> XrfResult<Option<Ltx>> {
    let logical_path: String = Self::to_logical(path);

    // A wildcard include resolves only to names the VFS holds, and a named include that is absent is nothing to merge -
    // the same tolerance the filesystem source shows a config not yet generated from TypeScript.
    if self.vfs.scoped(self.scope).find(&logical_path)?.is_none() {
      return Ok(None);
    }

    self.read_ltx(&logical_path).map(Some)
  }

  fn describe(&self, path: &Path) -> String {
    format!("{} (logical)", Self::to_logical(path))
  }
}
