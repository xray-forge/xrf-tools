use serde::Serialize;

/// What a channel does outside its keys, `BEH_*` (`xrCore/Animation/Envelope.hpp`).
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ArchiveAnmBehavior {
  /// `BEH_RESET`: falls back to nothing.
  Reset,
  /// `BEH_CONSTANT`: holds the value of the nearest key, which is what everything shipped declares.
  Constant,
  /// `BEH_REPEAT`: plays the keyed range again from its start.
  Repeat,
  /// `BEH_OSCILLATE`: plays the keyed range again backwards.
  Oscillate,
  /// `BEH_OFFSET`: repeats, each pass starting where the last ended.
  Offset,
  /// `BEH_LINEAR`: carries on along the slope the end key leaves.
  Linear,
  /// A value the engine gives no name, kept as the number the file stores.
  Unnamed { value: u8 },
}

impl ArchiveAnmBehavior {
  /// The behaviour a stored byte names.
  pub const fn of(value: u8) -> Self {
    match value {
      0 => Self::Reset,
      1 => Self::Constant,
      2 => Self::Repeat,
      3 => Self::Oscillate,
      4 => Self::Offset,
      5 => Self::Linear,
      value => Self::Unnamed { value },
    }
  }
}

#[cfg(test)]
mod tests {
  use super::ArchiveAnmBehavior;

  #[test]
  fn each_behavior_is_the_one_the_engine_numbers() {
    assert_eq!(ArchiveAnmBehavior::of(0), ArchiveAnmBehavior::Reset);
    assert_eq!(ArchiveAnmBehavior::of(1), ArchiveAnmBehavior::Constant);
    assert_eq!(ArchiveAnmBehavior::of(5), ArchiveAnmBehavior::Linear);
  }

  #[test]
  fn a_value_the_engine_does_not_name_keeps_its_number() {
    assert_eq!(ArchiveAnmBehavior::of(9), ArchiveAnmBehavior::Unnamed { value: 9 });
  }
}
