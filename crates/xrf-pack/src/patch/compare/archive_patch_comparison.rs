use std::cmp::Ordering;

use xrf_error::XrfResult;
use xrf_job::{JobHandle, JobScope};
use xrf_vfs::XrayAsset;

use crate::patch::PATCH_PHASE_COMPARE;
use crate::patch::compare::{ArchivePatchChange, ArchivePatchSide};
use crate::patch::config::ArchivePatchScope;
use crate::patch::world::{ArchivePatchChecksum, ArchivePatchRole, ArchivePatchWorld};

/// What two mounted worlds differ by, and how much of them did not.
#[derive(Debug, Default)]
pub(crate) struct ArchivePatchComparison {
  pub(crate) added: Vec<ArchivePatchChange>,
  pub(crate) modified: Vec<ArchivePatchChange>,
  pub(crate) removed: Vec<ArchivePatchChange>,
  pub(crate) unchanged: usize,
  /// Entries whose payload had to be read to classify them, which is what the cheap path avoided.
  pub(crate) payloads_read: usize,
  /// What each side offered before anything was compared, so a side holding nothing can be named as such.
  base_listed: usize,
  target_listed: usize,
}

impl ArchivePatchComparison {
  /// Compare two worlds, in one pass over both.
  ///
  /// Both sides arrive sorted by engine identity, which [`ArchivePatchWorld::list_scoped`] gets from the VFS for
  /// free, so this is a merge rather than a lookup per entry: neither side's name table is duplicated into a map,
  /// which at two mounted installations is the difference that decides the run's peak.
  ///
  /// # Errors
  ///
  /// Returns the read error of any payload a decision needed.
  pub(crate) fn of(
    base: &ArchivePatchWorld,
    target: &ArchivePatchWorld,
    scope: &ArchivePatchScope,
    job: &JobHandle,
    is_verifying_payload: bool,
  ) -> XrfResult<Self> {
    let base_entries: Vec<XrayAsset> = base.list_scoped(scope);
    let target_entries: Vec<XrayAsset> = target.list_scoped(scope);

    let comparing: JobScope = job.enter(PATCH_PHASE_COMPARE, Some(target_entries.len() as u64));
    let mut comparison: Self = Self {
      base_listed: base_entries.len(),
      target_listed: target_entries.len(),
      ..Self::default()
    };
    let (mut left, mut right) = (0usize, 0usize);

    while let Some(order) = Self::next(&base_entries[left..], &target_entries[right..]) {
      if job.is_cancelled() {
        break;
      }

      match order {
        Ordering::Less => {
          let entry: &XrayAsset = &base_entries[left];

          comparison
            .removed
            .push(ArchivePatchChange::removed(name_of(entry), base.to_side(entry)));

          left += 1;
        }
        Ordering::Greater => {
          let entry: &XrayAsset = &target_entries[right];

          comparison
            .added
            .push(ArchivePatchChange::added(name_of(entry), target.to_side(entry)));

          right += 1;
          comparing.advance();
        }
        Ordering::Equal => {
          comparison.classify_pair(
            base,
            target,
            &base_entries[left],
            &target_entries[right],
            is_verifying_payload,
          )?;

          left += 1;
          right += 1;
          comparing.advance();
        }
      }
    }

    Ok(comparison)
  }

  /// Everything the patch will carry, in the order the engine's own table iterates.
  ///
  /// Added and modified merge back into one sorted run rather than being concatenated: both came out of one ordered
  /// pass, and the packer's registration keys by engine name, so handing it a sorted run means the volume it writes
  /// does not depend on which class an entry landed in.
  pub(crate) fn to_carried_names(&self) -> Vec<&str> {
    let mut names: Vec<&str> = self
      .added
      .iter()
      .chain(self.modified.iter())
      .map(|change| change.name.as_str())
      .collect();

    names.sort_unstable();

    names
  }

  pub(crate) fn get_carried_count(&self) -> usize {
    self.added.len() + self.modified.len()
  }

  /// What one side offered before anything was compared.
  ///
  /// Keyed by role rather than published as two fields, so a caller checking both walks the roles instead of pairing
  /// two arrays and trusting itself to keep them in the same order.
  pub(crate) const fn get_listed_count(&self, role: ArchivePatchRole) -> usize {
    match role {
      ArchivePatchRole::Base => self.base_listed,
      ArchivePatchRole::Target => self.target_listed,
    }
  }

  /// Which side owns the next entry, or `None` once both are exhausted.
  fn next(base: &[XrayAsset], target: &[XrayAsset]) -> Option<Ordering> {
    match (base.first(), target.first()) {
      (Some(base), Some(target)) => Some(name_of(base).cmp(name_of(target))),
      (Some(_), None) => Some(Ordering::Less),
      (None, Some(_)) => Some(Ordering::Greater),
      (None, None) => None,
    }
  }

  /// Decide one entry both sides hold, spending as little as the pair allows.
  ///
  /// The ladder is the point. Size settles every pair that differs in length and costs nothing; only a pair of equal
  /// size needs a checksum, which an archive answers from its name table and a loose file answers by being read; and
  /// only a caller who distrusts CRC32 pays for the payload comparison on top. So an archive-to-archive run reads no
  /// payload at all, and a loose side reads only the files whose counterpart is the same size.
  fn classify_pair(
    &mut self,
    base: &ArchivePatchWorld,
    target: &ArchivePatchWorld,
    base_entry: &XrayAsset,
    target_entry: &XrayAsset,
    is_verifying_payload: bool,
  ) -> XrfResult<()> {
    let name: &str = name_of(target_entry);
    let base_side: ArchivePatchSide = base.to_side(base_entry);
    let target_side: ArchivePatchSide = target.to_side(target_entry);

    if base_side.size == target_side.size && self.reads_alike(base, target, name, is_verifying_payload)? {
      self.unchanged += 1;

      return Ok(());
    }

    self
      .modified
      .push(ArchivePatchChange::modified(name, base_side, target_side));

    Ok(())
  }

  /// Whether two same-sized entries hold the same payload.
  fn reads_alike(
    &mut self,
    base: &ArchivePatchWorld,
    target: &ArchivePatchWorld,
    name: &str,
    is_verifying_payload: bool,
  ) -> XrfResult<bool> {
    let base_checksum: ArchivePatchChecksum = base.read_checksum(name)?;
    let target_checksum: ArchivePatchChecksum = target.read_checksum(name)?;

    if base_checksum.is_hashed() || target_checksum.is_hashed() {
      self.payloads_read += 1;
    }

    if base_checksum.get_value() != target_checksum.get_value() {
      return Ok(false);
    }

    // Equal size and equal CRC32 is what the engine itself trusts on every decompression, so proving it costs two more
    // full reads and is asked for rather than assumed.
    if is_verifying_payload {
      return Ok(base.read_bytes(name)? == target.read_bytes(name)?);
    }

    Ok(true)
  }
}

/// The engine identity an entry is compared and reported under.
fn name_of(asset: &XrayAsset) -> &str {
  asset.get_logical_path().as_str()
}
