use std::collections::BTreeMap;
use std::path::PathBuf;

use xrf_error::{XrfError, XrfResult};

use crate::change::xray_disk_swap::XrayDiskSwap;
use crate::change::xray_transaction::XrayTransactionWrite;
use crate::{
  XrayAssetSource, XrayChange, XrayChangeKind, XrayChangeSet, XrayLookupScope, XrayMount, XrayMountId, XrayRevision,
  XraySourceKind, XrayTransaction, XrayVfs,
};

impl XrayVfs {
  /// Writes a transaction to disk and answers the snapshot it leaves, with what it changed: every file or none.
  ///
  /// # Errors
  ///
  /// Returns an error, with nothing on disk changed, when what the transaction expected moved, a path cannot be placed
  /// in a writable mount, a delete names a path no loose file answers for, or a file cannot be staged or swapped.
  pub fn commit(&self, transaction: &XrayTransaction) -> XrfResult<(XrayVfs, XrayChangeSet)> {
    transaction.check_expected(self)?;

    let mut swap: XrayDiskSwap = XrayDiskSwap::default();
    let mut changes: Vec<XrayChange> = Vec::with_capacity(transaction.get_writes().len());

    // todo: Keep what each write replaces (the bytes the winner held, or that none did), so an undo can commit it back.
    for write in transaction.get_writes() {
      let (change, target): (XrayChange, PathBuf) = self.place_write(transaction.get_scopes(), write)?;

      swap.stage(target, write.bytes.as_deref())?;
      changes.push(change);
    }

    swap.swap()?;

    self.apply(changes)
  }

  /// Answers the snapshot that `changes`, already on disk, leave, writing nothing: the seam a watcher or a refresh from
  /// disk uses, and the second half of every commit.
  ///
  /// # Errors
  ///
  /// Returns an error, with nothing this snapshot retains dropped, when a change names a mount this snapshot does not
  /// have, a path outside the mount's base, or an archive, whose volumes cannot take a change in place.
  pub fn apply(&self, changes: Vec<XrayChange>) -> XrfResult<(XrayVfs, XrayChangeSet)> {
    // todo: Let a change carry bytes, so an edit to an open buffer applies in memory with nothing on disk written.
    // Nothing changed is no change: the project revision moves only with a path that did.
    if changes.is_empty() {
      return Ok((self.fork(), XrayChangeSet::new(self.get_revision(), changes)));
    }

    let revision: XrayRevision = self.get_revision().next();
    let mut next: XrayVfs = self.fork();
    let mut changed: Vec<(&XrayMount, &str)> = Vec::with_capacity(changes.len());
    let mut structural: BTreeMap<XrayMountId, (Vec<&str>, Vec<&str>)> = BTreeMap::new();

    for change in &changes {
      let (mount, source_path): (&XrayMount, &str) = self.locate_change(change)?;
      let path: &str = change.get_path().as_str();

      next.get_revisions_mut().set_file(mount.get_id(), path, revision);
      changed.push((mount, source_path));

      if change.is_structural() {
        next
          .get_revisions_mut()
          .set_directories_above(mount.get_id(), path, revision);

        let (created, deleted) = structural.entry(mount.get_id()).or_default();

        match change.get_kind() {
          XrayChangeKind::Created => created.push(source_path),
          XrayChangeKind::Deleted => deleted.push(source_path),
          XrayChangeKind::Modified => {}
        }
      }
    }

    for (id, (created, deleted)) in structural {
      let source: Box<dyn XrayAssetSource> = self.get_mount(id)?.get_source().with_changes(&created, &deleted)?;

      next.replace_source(id, source);
    }

    // Only once the next snapshot stands: the lineage shares one cache, so this one's readers lose these values too.
    for (mount, source_path) in changed {
      next.forget_retained(mount, source_path);
    }

    next.set_revision(revision);

    Ok((next, XrayChangeSet::new(revision, changes)))
  }

  /// The loose mount a change names and the changed path in its source.
  fn locate_change<'a>(&'a self, change: &'a XrayChange) -> XrfResult<(&'a XrayMount, &'a str)> {
    let mount: &XrayMount = self.get_mount(change.get_mount())?;
    let path: &str = change.get_path().as_str();

    if mount.get_kind() == XraySourceKind::Archive {
      // todo: open a changed volume set again and move every entry it holds, for a watcher that sees one change.
      return Err(XrfError::new_not_implemented_error(format!(
        "cannot apply a change to '{path}': archive '{}' cannot take one in place",
        mount.get_label()
      )));
    }

    let source_path: &str = mount.to_source_path(path).ok_or_else(|| {
      XrfError::new_asset_error(format!(
        "cannot apply a change to '{path}': it falls outside the base of mount '{}'",
        mount.get_label()
      ))
    })?;

    Ok((mount, source_path))
  }

  /// Where a transaction's write lands, as the change it makes and the file it replaces, creates or deletes.
  fn place_write(&self, scopes: &[XrayLookupScope], write: &XrayTransactionWrite) -> XrfResult<(XrayChange, PathBuf)> {
    let path: &str = write.path.as_str();
    let winner: Option<(&XrayMount, &str)> = scopes.iter().find_map(|scope| self.get_winner_in_scope(scope, path));

    let mount: &XrayMount = match (winner, &write.bytes) {
      (Some((mount, _)), _) if mount.is_writable() => mount,
      (_, None) => {
        return Err(XrfError::new_asset_error(format!(
          "cannot delete '{path}': no writable loose mount the search reaches answers for it"
        )));
      }
      (_, Some(_)) => self.find_writable_for(scopes, path)?,
    };

    let source_path: &str = mount
      .to_source_path(path)
      .expect("a mount answering for a path, or chosen for it, holds it under its base");
    let target: PathBuf = mount.get_source().get_writable_path(source_path).ok_or_else(|| {
      XrfError::new_asset_error(format!(
        "cannot write '{path}': mount '{}' keeps no file on disk for it",
        mount.get_label()
      ))
    })?;
    let change: XrayChange = match (&write.bytes, mount.get_source().contains(source_path)) {
      (None, _) => XrayChange::deleted(mount.get_id(), path)?,
      (Some(_), true) => XrayChange::modified(mount.get_id(), path)?,
      (Some(_), false) => XrayChange::created(mount.get_id(), path)?,
    };

    Ok((change, target))
  }

  /// The first writable mount the scopes reach whose base holds `path`, in search order.
  fn find_writable_for<'a>(&'a self, scopes: &[XrayLookupScope], path: &str) -> XrfResult<&'a XrayMount> {
    scopes
      .iter()
      .filter(|scope| {
        scope
          .get_prefix()
          .is_none_or(|prefix| crate::path::is_component_prefix(path, prefix))
      })
      .flat_map(|scope| self.mounts_in(scope))
      .find(|mount| mount.is_writable() && mount.to_source_path(path).is_some())
      .ok_or_else(|| {
        XrfError::new_asset_error(format!(
          "cannot write '{path}': no writable loose mount the search reaches can hold it"
        ))
      })
  }

  fn get_mount(&self, id: XrayMountId) -> XrfResult<&XrayMount> {
    self
      .get_mounts()
      .get(id.0)
      .ok_or_else(|| XrfError::new_asset_error(format!("no mount {id:?} in this snapshot")))
  }
}
