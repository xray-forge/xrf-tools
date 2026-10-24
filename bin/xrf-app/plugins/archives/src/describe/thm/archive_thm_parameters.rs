use serde::Serialize;
use xrf_thm::{ThmTextureFlag, ThmTextureParamChunk};

/// The conversion parameters a descriptor carries, `THM_CHUNK_TEXTUREPARAM`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveThmParameters {
  pub format_label: String,
  pub format: u32,
  pub mip_filter_label: String,
  pub mip_filter: u32,
  pub border_color: u32,
  pub fade_color: u32,
  pub fade_amount: u32,
  pub width: u32,
  pub height: u32,
  /// Every bit the SDK names, in bit order, set or not.
  pub flags: Vec<ArchiveThmFlag>,
  /// Bits the word carries that the SDK has no name for.
  pub unnamed_flags: u32,
}

/// One bit of the texture param flag word.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveThmFlag {
  /// SDK identifier, which is the name an author of a `.thm` would recognise.
  pub label: String,
  pub is_set: bool,
}

impl ArchiveThmParameters {
  pub fn of(parameters: &ThmTextureParamChunk) -> Self {
    Self {
      format_label: parameters.format.label(),
      format: parameters.format.into(),
      mip_filter_label: parameters.mip_filter.label(),
      mip_filter: parameters.mip_filter.into(),
      border_color: parameters.border_color,
      fade_color: parameters.fade_color,
      fade_amount: parameters.fade_amount,
      width: parameters.width,
      height: parameters.height,
      flags: ThmTextureFlag::NAMED
        .into_iter()
        .map(|flag| ArchiveThmFlag {
          label: flag.label().to_owned(),
          is_set: parameters.flags.has(flag),
        })
        .collect(),
      unnamed_flags: parameters.flags.unnamed(),
    }
  }
}
