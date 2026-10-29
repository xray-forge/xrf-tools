use serde::Serialize;

/// Whether a bump's relief lies where its diffuse's features do.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BumpAlignmentVerdict {
  /// The relief follows the diffuse's edges well past chance: one set, authored together.
  Aligned,
  /// Some agreement, not enough to call: a soft or periodic texture, or a bump only loosely drawn from its diffuse.
  Doubtful,
  /// Little more agreement than the bump shifted along the tile: a diffuse and a bump from different sets, as a pack
  /// installed in part leaves them, or a descriptor borrowing an unlike texture's bump.
  Mismatched,
  /// One of the two carries next to nothing to compare: a flat bump, or a diffuse of one colour.
  Featureless,
}

/// The lifts are read off stock Anomaly (2026-10-30): its sets' lifts run from about -0.08 to 0.6, and the sets under
/// 0.03 that were looked at hold other art in their bump (other stones, a weapon's parts moved and relabelled).
impl BumpAlignmentVerdict {
  /// The lift over chance an aligned set reaches at least.
  pub const ALIGNED_LIFT: f32 = 0.1;
  /// The lift over chance a mismatched set stays under.
  pub const MISMATCHED_LIFT: f32 = 0.03;

  /// The verdict as a report spells it.
  pub fn label(self) -> &'static str {
    match self {
      Self::Aligned => "aligned",
      Self::Doubtful => "doubtful",
      Self::Mismatched => "mismatched",
      Self::Featureless => "featureless",
    }
  }

  /// The verdict of a correlation `lift` over the shifted bump's.
  pub fn of(lift: f32) -> Self {
    if lift >= Self::ALIGNED_LIFT {
      Self::Aligned
    } else if lift < Self::MISMATCHED_LIFT {
      Self::Mismatched
    } else {
      Self::Doubtful
    }
  }
}
