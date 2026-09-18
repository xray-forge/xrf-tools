use serde::Serialize;
use xrf_thm::{ThmFile, ThmTextureType};

/// The kind of texture a descriptor describes, `STextureParams::ETType`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveThmTextureType {
  /// Engine token for the type, or the raw number for one the SDK does not name.
  pub label: String,
  pub value: u32,
  /// Whether `CTextureDescrMngr::LoadTHM` reads the bump, detail and material of a descriptor of this type at all.
  pub is_read_by_engine: bool,
  /// Whether the file declares a type, or the engine's zeroed default is what applies.
  pub is_declared: bool,
}

impl ArchiveThmTextureType {
  /// The type gate a descriptor presents, declared or defaulted.
  pub fn of(file: &ThmFile) -> Self {
    let texture_type: ThmTextureType = file.texture_type();

    Self {
      label: texture_type.label(),
      value: texture_type.into(),
      is_read_by_engine: texture_type.is_described_by_engine(),
      is_declared: file.texture_type.is_some(),
    }
  }
}
