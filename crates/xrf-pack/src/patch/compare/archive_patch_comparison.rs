use std::cmp::Ordering;

use xrf_error::XrfResult;
use xrf_job::{JobHandle, JobScope};
use xrf_vfs::XrayAsset;

use crate::patch::PATCH_PHASE_COMPARE;
use crate::patch::compare::{ArchivePatchChange, ArchivePatchOrigins, ArchivePatchSide};
use crate::patch::config::ArchivePatchScope;
use crate::patch::world::{ArchivePatchChecksum, ArchivePatchRole, ArchivePatchWorld};

/// Classified differences and counts for two mounted roots.
#[derive(Debug, Default)]
pub(crate) struct ArchivePatchComparison {
  pub(crate) added: Vec<ArchivePatchChange>,
  pub(crate) modified: Vec<ArchivePatchChange>,
  pub(crate) unchanged: usize,
  /// Entry pairs requiring a computed checksum.
  pub(crate) payloads_read: usize,
  /// Every volume set and loose root the two sides read from, which each side names by index.
  pub(crate) origins: ArchivePatchOrigins,
  /// Base entries in scope before comparison.
  base_listed: usize,
  target_listed: usize,
}

impl ArchivePatchComparison {
  /// Compares sorted engine identities in one pass without building lookup maps.
  ///
  /// # Errors
  ///
  /// Propagates payload read errors.
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
          // A patch adds to and overrides the base and says nothing about the rest of it, so an entry only the base
          // holds is a file nobody touched rather than a finding. At an installation that is the whole game.
          left += 1;
        }
        Ordering::Greater => {
          let entry: &XrayAsset = &target_entries[right];
          let side: ArchivePatchSide = target.to_side(entry, &mut comparison.origins);

          comparison.added.push(ArchivePatchChange::added(name_of(entry), side));

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

  /// Returns added and modified engine names in sorted order.
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

  /// Total unpacked size of added and modified target entries.
  pub(crate) fn get_carried_size(&self) -> u64 {
    self
      .added
      .iter()
      .chain(self.modified.iter())
      .filter_map(|change| change.target.as_ref())
      .map(|side| side.size)
      .sum()
  }

  /// Returns the number of entries in scope on the requested side.
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

  /// Classifies a shared entry by size, then checksum, with optional byte-for-byte verification.
  fn classify_pair(
    &mut self,
    base: &ArchivePatchWorld,
    target: &ArchivePatchWorld,
    base_entry: &XrayAsset,
    target_entry: &XrayAsset,
    is_verifying_payload: bool,
  ) -> XrfResult<()> {
    let name: &str = name_of(target_entry);
    let base_side: ArchivePatchSide = base.to_side(base_entry, &mut self.origins);
    let target_side: ArchivePatchSide = target.to_side(target_entry, &mut self.origins);

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
