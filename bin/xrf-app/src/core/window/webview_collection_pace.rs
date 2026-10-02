use serde::{Deserialize, Serialize};

/// How early V8 starts marking for its next major collection: earlier makes more collections, each freeing less at once.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WebviewCollectionPace {
  /// V8's own pace.
  #[default]
  Default,
  /// Marking starts at half V8's own point.
  Earlier,
  /// Marking starts at a quarter of it.
  Frequent,
}

impl WebviewCollectionPace {
  /// `--incremental-marking-soft-trigger`, the percentage of V8's own start point marking starts at, where it moves it.
  pub fn soft_trigger(self) -> Option<u8> {
    match self {
      Self::Default => None,
      Self::Earlier => Some(50),
      Self::Frequent => Some(25),
    }
  }
}
