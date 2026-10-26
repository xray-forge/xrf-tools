use serde::Serialize;
use xrf_thm::ThmTextureType;

/// What the SDK's editor changes when a descriptor is given one texture type, `STextureParams::OnTypeChange`, so the
/// form makes the same change when its type does.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureTypeChange {
  pub texture_type: u32,
  /// Bits cleared from the flag word, before the set ones are set.
  pub cleared_flags: u32,
  pub set_flags: u32,
  /// The format the type brings, or none where the descriptor keeps its own.
  pub format: Option<u32>,
  /// The mip filter the type brings, or none where the descriptor keeps its own.
  pub mip_filter: Option<u32>,
}

impl TextureTypeChange {
  /// The change for every type the SDK names, in the order it lists them.
  pub fn of_all() -> Vec<Self> {
    ThmTextureType::NAMED
      .into_iter()
      .map(|texture_type| Self {
        texture_type: texture_type.into(),
        cleared_flags: texture_type.get_cleared_flags().raw(),
        set_flags: texture_type.get_set_flags().raw(),
        format: texture_type.get_format().map(u32::from),
        mip_filter: texture_type.get_mip_filter().map(u32::from),
      })
      .collect()
  }
}
