use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_utils::{decode_bytes_to_string, new_windows1251_encoder};
use xrf_vfs::{XrayLogicalPath, XrayLookupScope, XrayVfs};

use crate::Ltx;
use crate::file::include::LtxIncludeConvertor;
use crate::file::include_source::LtxIncludeSource;

/// Resolves and reads includes through a mounted VFS.
///
/// This is what lets configs be read out of an installation, where they live inside `db\configs` volumes. A wildcard include
/// cannot be answered by `read_dir` there, so it becomes prefix enumeration over the VFS instead - the same operation, asked
/// of a logical tree rather than a directory.
pub(crate) struct LtxIncludeVfsSource<'a> {
  vfs: &'a XrayVfs,
  scope: &'a XrayLookupScope,
}

impl<'a> LtxIncludeVfsSource<'a> {
  pub fn new(vfs: &'a XrayVfs, scope: &'a XrayLookupScope) -> Self {
    Self { scope, vfs }
  }

  /// Reads and parses one logical path, with its logical location recorded so nested includes resolve against it.
  pub fn read_ltx(&self, logical_path: &str) -> XrfResult<Ltx> {
    let bytes: Vec<u8> = self.vfs.scoped(self.scope).read_bytes(logical_path)?;
    let contents: String = decode_bytes_to_string(&bytes, new_windows1251_encoder())?;
    let mut ltx: Ltx = Ltx::read_from_str(&contents)?;
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

  /// A logical path as the VFS spells it.
  ///
  /// `PathBuf` may have normalized separators for the host, so this converts back rather than trusting `to_string_lossy`.
  fn to_logical(path: &Path) -> String {
    path.to_string_lossy().replace('/', "\\")
  }
}

impl LtxIncludeSource for LtxIncludeVfsSource<'_> {
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
