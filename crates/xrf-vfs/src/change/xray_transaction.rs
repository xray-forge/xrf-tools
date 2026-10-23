use xrf_error::{XrfError, XrfResult};

use crate::{XrayInputRecord, XrayLogicalPath, XrayLookupScope, XraySearch, XraySearchKey, XrayVfs};

/// One gesture's writes and deletes, committed all at once or not at all through [`XrayVfs::commit`].
// todo: Tag a transaction with the editor or document that issued it, so undoing it is that editor's alone.
#[derive(Clone, Debug)]
pub struct XrayTransaction {
  key: XraySearchKey,
  writes: Vec<XrayTransactionWrite>,
  expected: Option<XrayInputRecord>,
}

/// One path a transaction changes: its new bytes, or `None` to delete it.
#[derive(Clone, Debug)]
pub(crate) struct XrayTransactionWrite {
  pub(crate) path: XrayLogicalPath,
  pub(crate) bytes: Option<Vec<u8>>,
}

impl XrayTransaction {
  /// An empty transaction addressing paths as `search` reads them.
  pub fn new(search: &XraySearch) -> Self {
    Self {
      key: XraySearchKey::of(search.get_steps()),
      writes: Vec::new(),
      expected: None,
    }
  }

  /// Writes `bytes` at a logical path, replacing an earlier write or delete of it in this transaction.
  ///
  /// # Errors
  ///
  /// Returns an error when `path` is not a valid X-Ray logical path.
  pub fn with_write(self, path: &str, bytes: impl Into<Vec<u8>>) -> XrfResult<Self> {
    self.with(path, Some(bytes.into()))
  }

  /// Deletes the loose file a logical path resolves to, replacing an earlier write of it in this transaction.
  ///
  /// # Errors
  ///
  /// Returns an error when `path` is not a valid X-Ray logical path.
  pub fn with_delete(self, path: &str) -> XrfResult<Self> {
    self.with(path, None)
  }

  /// Refuses the commit when any input `record` holds answers differently by then: what the author read moved under it.
  pub fn with_expected(mut self, record: &XrayInputRecord) -> Self {
    self.expected = Some(record.clone());

    self
  }

  pub fn is_empty(&self) -> bool {
    self.writes.is_empty()
  }

  pub(crate) fn get_scopes(&self) -> &[XrayLookupScope] {
    self.key.get_scopes()
  }

  pub(crate) fn get_writes(&self) -> &[XrayTransactionWrite] {
    &self.writes
  }

  /// Fails when what the author read no longer answers as it did in `vfs`.
  pub(crate) fn check_expected(&self, vfs: &XrayVfs) -> XrfResult<()> {
    let Some(expected) = &self.expected else {
      return Ok(());
    };

    match expected.list_moved_in(vfs).first() {
      None => Ok(()),
      Some(input) => Err(XrfError::new_asset_error(format!(
        "refusing a stale change: {input} changed since it was read"
      ))),
    }
  }

  fn with(mut self, path: &str, bytes: Option<Vec<u8>>) -> XrfResult<Self> {
    let path: XrayLogicalPath = XrayLogicalPath::new(path)?;

    match self.writes.iter_mut().find(|write| write.path == path) {
      Some(write) => write.bytes = bytes,
      None => self.writes.push(XrayTransactionWrite { path, bytes }),
    }

    Ok(self)
  }
}
