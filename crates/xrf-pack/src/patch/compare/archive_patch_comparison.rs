use std::cmp::Ordering;

use rayon::ThreadPoolBuilder;
use rayon::prelude::*;

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
  /// Propagates payload read errors, reporting the first in merge order so a failure does not depend on scheduling.
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
    let mut pending: Vec<ArchivePatchPending> = Vec::new();

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
          comparison.plan_pair(base, target, &base_entries[left], &target_entries[right], &mut pending);

          left += 1;
          right += 1;
        }
      }
    }

    comparison.settle(base, target, pending, &comparing, job, is_verifying_payload)?;

    Ok(comparison)
  }

  /// Decide every equal-sized pair the merge deferred, in parallel, and fold the answers back in merge order.
  ///
  /// # Errors
  ///
  /// Returns the first read error in merge order.
  fn settle(
    &mut self,
    base: &ArchivePatchWorld,
    target: &ArchivePatchWorld,
    pending: Vec<ArchivePatchPending>,
    comparing: &JobScope,
    job: &JobHandle,
    is_verifying_payload: bool,
  ) -> XrfResult<()> {
    let decide_all = || -> Vec<XrfResult<ArchivePatchDecision>> {
      // An indexed collection, so rayon hands the answers back in the order the merge queued them.
      pending
        .par_iter()
        .map(|entry| {
          if job.is_cancelled() {
            return Ok(ArchivePatchDecision::cancelled());
          }

          let decision: XrfResult<ArchivePatchDecision> = Self::decide(base, target, &entry.name, is_verifying_payload);

          comparing.advance();

          decision
        })
        .collect()
    };

    // Its own pool rather than the global one, so the bound is this phase's and no other work inherits it. A pool
    // that cannot be built is not worth failing a comparison over: the same decisions are made sequentially.
    let decided: Vec<XrfResult<ArchivePatchDecision>> = match ThreadPoolBuilder::new()
      .num_threads(DECISION_CONCURRENCY)
      .build()
    {
      Ok(pool) => pool.install(decide_all),
      Err(_) => decide_all(),
    };

    for (entry, decision) in pending.into_iter().zip(decided) {
      let decision: ArchivePatchDecision = decision?;

      if decision.is_payload_read {
        self.payloads_read += 1;
      }

      if decision.is_alike {
        self.unchanged += 1;
      } else {
        self
          .modified
          .push(ArchivePatchChange::modified(&entry.name, entry.base, entry.target));
      }
    }

    Ok(())
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

  /// Classifies a shared entry by size, deferring anything a size cannot settle.
  ///
  /// Interning both origins happens here rather than in the parallel phase: it is the comparison's only shared
  /// mutable state, and the merge is already sequential.
  fn plan_pair(
    &mut self,
    base: &ArchivePatchWorld,
    target: &ArchivePatchWorld,
    base_entry: &XrayAsset,
    target_entry: &XrayAsset,
    pending: &mut Vec<ArchivePatchPending>,
  ) {
    let name: &str = name_of(target_entry);
    let base_side: ArchivePatchSide = base.to_side(base_entry, &mut self.origins);
    let target_side: ArchivePatchSide = target.to_side(target_entry, &mut self.origins);

    if base_side.size == target_side.size {
      pending.push(ArchivePatchPending {
        name: name.to_owned(),
        base: base_side,
        target: target_side,
      });

      return;
    }

    self
      .modified
      .push(ArchivePatchChange::modified(name, base_side, target_side));
  }

  /// Whether two same-sized entries hold the same payload, and what deciding it cost.
  fn decide(
    base: &ArchivePatchWorld,
    target: &ArchivePatchWorld,
    name: &str,
    is_verifying_payload: bool,
  ) -> XrfResult<ArchivePatchDecision> {
    let base_checksum: ArchivePatchChecksum = base.read_checksum(name)?;
    let target_checksum: ArchivePatchChecksum = target.read_checksum(name)?;
    let is_payload_read: bool = base_checksum.is_hashed() || target_checksum.is_hashed();

    if base_checksum.get_value() != target_checksum.get_value() {
      return Ok(ArchivePatchDecision {
        is_alike: false,
        is_payload_read,
      });
    }

    // Equal size and equal CRC32 is what the engine itself trusts on every decompression, so proving it costs two more
    // full reads and is asked for rather than assumed.
    let is_alike: bool = if is_verifying_payload {
      base.read_bytes(name)? == target.read_bytes(name)?
    } else {
      true
    };

    Ok(ArchivePatchDecision {
      is_alike,
      is_payload_read,
    })
  }
}

/// How many payloads a comparison reads at once.
///
/// Deciding a pair holds one payload per side in memory, so peak memory tracks this rather than the entry count. On a
/// 32-thread machine the unbounded pool read a 21 GB installation in 5.1 s against 14.0 s sequential, and took peak
/// RSS from 248 MB to 1086 MB — most of the wall-clock win comes from having several reads in flight at all, and the
/// rest of the cores only buy memory. Bounded here so a workstation and a build agent behave the same way.
const DECISION_CONCURRENCY: usize = 8;

/// An equal-sized pair the merge could not settle, waiting for a checksum.
struct ArchivePatchPending {
  name: String,
  base: ArchivePatchSide,
  target: ArchivePatchSide,
}

/// What deciding one pending pair concluded, and what it cost.
struct ArchivePatchDecision {
  is_alike: bool,
  is_payload_read: bool,
}

impl ArchivePatchDecision {
  /// What a cancelled run reports for a pair it never looked at: unchanged, and nothing read.
  ///
  /// A cancellation already discards the publication, so the value only has to be one the fold can carry without
  /// inventing a difference nobody measured.
  const fn cancelled() -> Self {
    Self {
      is_alike: true,
      is_payload_read: false,
    }
  }
}

/// The engine identity an entry is compared and reported under.
fn name_of(asset: &XrayAsset) -> &str {
  asset.get_logical_path().as_str()
}
