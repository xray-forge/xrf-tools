//! The names the SDK gives the numbers a descriptor stores.

use serde::Serialize;
use xrf_db::{ThmBumpMode, ThmFormat, ThmMaterial, ThmMipFilter, ThmTextureFlag, ThmTextureType};

/// One value a descriptor field can take, under the name the SDK gives it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureVocabularyEntry {
  /// The number stored in the file.
  pub value: u32,
  /// The SDK's own identifier for it, which is the name an author of a `.thm` would recognise.
  pub label: String,
}

/// One bit of the `STextureParams` flag word.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureFlagEntry {
  /// The bit, as a mask rather than an index, because the SDK's own bits are not contiguous.
  pub bit: u32,
  pub label: String,
}

/// Every named value the descriptor form's numeric fields can take.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureVocabulary {
  /// The gate `LoadTHM` reads before anything else.
  pub texture_types: Vec<TextureVocabularyEntry>,
  pub formats: Vec<TextureVocabularyEntry>,
  /// Fifteen values, of which fourteen are kernels and `Advanced` is the SDK's own chain.
  pub mip_filters: Vec<TextureVocabularyEntry>,
  pub materials: Vec<TextureVocabularyEntry>,
  pub bump_modes: Vec<TextureVocabularyEntry>,
  /// The twelve bits the SDK names, in bit order. A word may carry others, and those have no name to show.
  pub flags: Vec<TextureFlagEntry>,
  /// The bump mode that makes the engine bind a pair.
  pub bump_mode_use: u32,
}

impl TextureVocabulary {
  /// The whole table, in the order each enum's own `NAMED` lists it.
  pub fn describe() -> Self {
    Self {
      texture_types: to_entries(ThmTextureType::NAMED, ThmTextureType::label),
      formats: to_entries(ThmFormat::NAMED, ThmFormat::label),
      mip_filters: to_entries(ThmMipFilter::NAMED, ThmMipFilter::label),
      materials: to_entries(ThmMaterial::NAMED, ThmMaterial::label),
      bump_modes: to_entries(ThmBumpMode::NAMED, ThmBumpMode::label),
      flags: to_flag_entries(),
      bump_mode_use: ThmBumpMode::Use.into(),
    }
  }
}

/// One enum's named values, each paired with the number it is stored as.
fn to_entries<T: Copy + Into<u32>>(
  named: impl IntoIterator<Item = T>,
  label: fn(T) -> String,
) -> Vec<TextureVocabularyEntry> {
  named
    .into_iter()
    .map(|value| TextureVocabularyEntry {
      value: value.into(),
      label: label(value),
    })
    .collect()
}

/// The flag word's named bits, which are the one field addressed by mask rather than by value.
fn to_flag_entries() -> Vec<TextureFlagEntry> {
  ThmTextureFlag::NAMED
    .into_iter()
    .map(|flag| TextureFlagEntry {
      bit: flag.bit(),
      label: flag.label().to_owned(),
    })
    .collect()
}
