use std::path::PathBuf;

use xrf_error::{XrfError, XrfResult};

use crate::change::transaction::{XrayTransactionTarget, XrayTransactionWrite};
use crate::{XrayInputRecord, XrayLogicalPath, XrayLookupScope, XraySearch, XraySearchKey, XrayVfs};

/// One gesture's writes and deletes, committed all at once or not at all through [`XrayVfs::commit`].
// todo: Tag a transaction with the owner that issued it (the application's session scope), so undoing it is that
//  owner's alone.
#[derive(Clone, Debug)]
pub struct XrayTransaction {
  key: XraySearchKey,
  writes: Vec<XrayTransactionWrite>,
  expected: Option<XrayInputRecord>,
  is_in_place: bool,
}

impl XrayTransaction {
  /// An empty transaction addressing paths as `search` reads them.
  pub fn new(search: &XraySearch) -> Self {
    Self {
      key: XraySearchKey::of(search.get_steps()),
      writes: Vec::new(),
      expected: None,
      is_in_place: false,
    }
  }

  /// Writes `bytes` at a logical path, replacing an earlier write or delete of it in this transaction.
  ///
  /// # Errors
  ///
  /// Returns an error when `path` is not a valid X-Ray logical path.
  pub fn with_write(self, path: &str, bytes: impl Into<Vec<u8>>) -> XrfResult<Self> {
    Ok(self.with(
      XrayTransactionTarget::Path(XrayLogicalPath::new(path)?),
      Some(bytes.into()),
    ))
  }

  /// Writes `bytes` to the file at `file` on disk, in every writable loose mount of the search whose root holds it,
  /// whichever copy of its logical path the search answers with: for a writer that names files on disk. Refused at
  /// commit where no such mount holds it ([`XraySearch::holds_file`]).
  pub fn with_file_write(self, file: impl Into<PathBuf>, bytes: impl Into<Vec<u8>>) -> Self {
    self.with(XrayTransactionTarget::File(file.into()), Some(bytes.into()))
  }

  /// Deletes the loose file a logical path resolves to, replacing an earlier write of it in this transaction.
  ///
  /// # Errors
  ///
  /// Returns an error when `path` is not a valid X-Ray logical path.
  pub fn with_delete(self, path: &str) -> XrfResult<Self> {
    Ok(self.with(XrayTransactionTarget::Path(XrayLogicalPath::new(path)?), None))
  }

  /// Refuses the commit when any input `record` holds answers differently by then: what the author read moved under it.
  pub fn with_expected(mut self, record: &XrayInputRecord) -> Self {
    self.expected = Some(record.clone());

    self
  }

  /// Refuses the commit unless a writable loose copy answers for every logical path it writes, so each lands in the file
  /// the search answers with rather than in a new one created ahead of an archived copy.
  pub fn with_write_in_place(mut self) -> Self {
    self.is_in_place = true;

    self
  }

  pub fn is_empty(&self) -> bool {
    self.writes.is_empty()
  }

  pub(crate) fn is_in_place(&self) -> bool {
    self.is_in_place
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

    // Named by text order, since the record keeps no order of its own and a refusal should read the same every time.
    let moved: Vec<String> = expected.list_moved_in(vfs).iter().map(ToString::to_string).collect();

    match moved.iter().min() {
      None => Ok(()),
      Some(first) => Err(XrfError::new_asset_error(format!(
        "refusing a stale change: {} input(s) changed since they were read, {first} first",
        moved.len()
      ))),
    }
  }

  fn with(mut self, target: XrayTransactionTarget, bytes: Option<Vec<u8>>) -> Self {
    match self.writes.iter_mut().find(|write| write.target == target) {
      Some(write) => write.bytes = bytes,
      None => self.writes.push(XrayTransactionWrite { target, bytes }),
    }

    self
  }
}
