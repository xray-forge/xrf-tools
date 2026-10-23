use std::collections::{BTreeSet, HashMap};
use std::time::Instant;

use crate::cache::XrayStampedCopy;
use crate::{XrayChange, XrayFileStamp, XrayLookupScope, XrayMount, XrayMountId, XrayVfs};

impl XrayVfs {
  /// The loose copies the lineage retains values of, in the mounts `scopes` reach, whose file no longer looks as it did
  /// when it was read, each as the change [`Self::apply`] takes. Bounded by what is retained: nothing else is looked
  /// at, so a file changed behind every reader that nothing retains is not seen.
  // todo: Watch the mounted loose roots and apply each change as it lands, which also sees files created and deleted
  //  behind every reader, so an open need not look.
  pub(crate) fn list_changed_on_disk(&self, scopes: &[XrayLookupScope]) -> Vec<XrayChange> {
    let started: Instant = Instant::now();
    let mounts: HashMap<u64, &XrayMount> = scopes
      .iter()
      .flat_map(|scope| self.mounts_in(scope))
      .filter(|mount| mount.is_writable())
      .map(|mount| (mount.get_opening(), mount))
      .collect();
    let copies: Vec<XrayStampedCopy> = self.get_cache().list_stamped(|opening| mounts.contains_key(&opening));
    // One look per file, however many values were parsed from it.
    let mut looked: HashMap<(u64, &str), Option<XrayFileStamp>> = HashMap::new();
    let mut changed: BTreeSet<(XrayMountId, String)> = BTreeSet::new();

    for copy in &copies {
      let mount: &XrayMount = mounts[&copy.opening];
      let Ok(logical_path) = mount.to_logical_path(&copy.path) else {
        continue;
      };

      // A value read at an older revision is one no reader of this snapshot is served.
      if copy.revision != self.get_revisions().get_file(mount.get_id(), &logical_path) {
        continue;
      }

      let now: Option<XrayFileStamp> = *looked
        .entry((copy.opening, copy.path.as_str()))
        .or_insert_with(|| mount.get_source().get_stamp(&copy.path));

      if now != Some(copy.stamp) {
        changed.insert((mount.get_id(), logical_path.into_owned()));
      }
    }

    log::debug!(
      "Looked at {} retained loose file(s) on disk in {:.1?}: {} changed",
      looked.len(),
      started.elapsed(),
      changed.len()
    );

    changed
      .into_iter()
      .filter_map(|(mount, path)| XrayChange::modified(mount, &path).ok())
      .collect()
  }
}
