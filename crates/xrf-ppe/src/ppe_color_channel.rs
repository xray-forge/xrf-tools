use serde::{Deserialize, Serialize};

/// One channel of a colour parameter, by the index `CPostProcessColor::add_value` takes it at.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PpeColorChannel {
  Red,
  Green,
  Blue,
}

impl PpeColorChannel {
  /// Every channel, in the order the file stores them.
  pub const ALL: [Self; 3] = [Self::Red, Self::Green, Self::Blue];

  /// The name an author knows the channel by.
  pub const fn get_name(self) -> &'static str {
    match self {
      Self::Red => "red",
      Self::Green => "green",
      Self::Blue => "blue",
    }
  }
}
