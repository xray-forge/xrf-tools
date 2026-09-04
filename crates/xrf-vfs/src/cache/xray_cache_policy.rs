use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::asset::XrayAssetType;

/// What a mounted world is allowed to keep after parsing it.
///
/// Default-deny by kind, because an editor and a batch verifier want opposite answers: a preview reopens one model
/// constantly and wants it resident, while a sweep touches every texture in the installation exactly once and would
/// only pay to hold them. Naming the kinds makes the memory ceiling something a caller chose rather than something
/// inherited from whichever consumer was written first.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayCachePolicy {
  kinds: HashSet<XrayAssetType>,
  /// Last-resort ceiling on retained bytes, not a working budget.
  ///
  /// A run that reaches it is expected to be pathological, so the behaviour there is chosen to be safe rather than
  /// generous: the store stops retaining and hands the value back uncached.
  budget: Option<u64>,
}

impl XrayCachePolicy {
  /// Retains nothing; every read parses.
  pub fn none() -> Self {
    Self::default()
  }

  /// What a verification sweep benefits from.
  ///
  /// Motions and configs. A visual is enumerated once per sweep, so retaining it would cost a gigabyte to serve no
  /// second read, while shared animation banks are read once per referencing visual — the four Anomaly ships are read
  /// over nine hundred times each.
  pub fn verification() -> Self {
    Self::none().with(XrayAssetType::Omf).with(XrayAssetType::Ltx)
  }

  /// What a command that only reads configs benefits from.
  pub fn configs() -> Self {
    Self::none().with(XrayAssetType::Ltx)
  }

  /// What an editing session benefits from, where the same handful of assets is reopened constantly.
  pub fn editing() -> Self {
    Self::none()
      .with(XrayAssetType::Ogf)
      .with(XrayAssetType::Omf)
      .with(XrayAssetType::Dds)
      .with(XrayAssetType::Thm)
  }

  /// Extends a preset with one more kind.
  pub fn with(mut self, kind: XrayAssetType) -> Self {
    self.kinds.insert(kind);

    self
  }

  /// Caps retained bytes, measured as the source length of everything held.
  ///
  /// # Determinism
  ///
  /// A budget gives up schedule-independent cache statistics, so set one only where that is worth trading. Refusal
  /// depends on what is retained at the moment of an insert, and what a run can drop first depends on which values
  /// other threads still hold, through `Arc::strong_count`. Two runs over identical inputs can therefore refuse
  /// different values and report different `entries`, `bytes` and `refused`.
  ///
  /// Without a budget those counters describe the inputs alone, which is what lets a verification report be compared
  /// across runs and across worker counts. Nothing else in the store reintroduces the dependence: concurrent misses of
  /// one retained key are coordinated into a single load by [`XrayAssetCache::get_or_load`].
  ///
  /// [`XrayAssetCache::get_or_load`]: crate::XrayAssetCache::get_or_load
  pub fn with_budget(mut self, bytes: u64) -> Self {
    self.budget = Some(bytes);

    self
  }

  pub fn is_allowed(&self, kind: XrayAssetType) -> bool {
    self.kinds.contains(&kind)
  }

  pub fn get_budget(&self) -> Option<u64> {
    self.budget
  }

  /// Whether this policy can retain anything at all, so a caller can skip bookkeeping entirely.
  pub fn is_empty(&self) -> bool {
    self.kinds.is_empty()
  }
}
