use serde::Serialize;
use xrf_thm::ThmTextureFlag;

/// One bit of the `STextureParams` flag word.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureFlagEntry {
  /// The bit, as a mask rather than an index, because the SDK's own bits are not contiguous.
  pub bit: u32,
  pub label: String,
}

impl TextureFlagEntry {
  /// The flag word's named bits, which are the one field addressed by mask rather than by value.
  pub fn of_all() -> Vec<Self> {
    ThmTextureFlag::NAMED
      .into_iter()
      .map(|flag| Self {
        bit: flag.bit(),
        label: flag.label().to_owned(),
      })
      .collect()
  }
}
