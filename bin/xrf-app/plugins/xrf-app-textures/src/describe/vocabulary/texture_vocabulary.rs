use serde::Serialize;
use xrf_texture::GenerateBumpResult;
use xrf_thm::{ThmBumpMode, ThmFormat, ThmMaterial, ThmMipFilter, ThmTextureType};

use crate::describe::vocabulary::{TextureFlagEntry, TextureTypeChange, TextureVocabularyEntry};

/// Every named value the descriptor form's numeric fields can take.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
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
  /// What giving a descriptor each type changes beside the type, as the SDK's editor does.
  pub type_changes: Vec<TextureTypeChange>,
  /// The bump mode that makes the engine bind a pair.
  pub bump_mode_use: u32,
  /// The mean gloss below which a generated bump shows almost no specular response.
  pub minimum_gloss_power: f32,
}

impl TextureVocabulary {
  /// The whole table, in the order each enum's own `NAMED` lists it.
  pub fn describe() -> Self {
    Self {
      texture_types: TextureVocabularyEntry::of_all(ThmTextureType::NAMED, ThmTextureType::label),
      formats: TextureVocabularyEntry::of_all(ThmFormat::NAMED, ThmFormat::label),
      mip_filters: TextureVocabularyEntry::of_all(ThmMipFilter::NAMED, ThmMipFilter::label),
      materials: TextureVocabularyEntry::of_all(ThmMaterial::NAMED, ThmMaterial::label),
      bump_modes: TextureVocabularyEntry::of_all(ThmBumpMode::NAMED, ThmBumpMode::label),
      flags: TextureFlagEntry::of_all(),
      type_changes: TextureTypeChange::of_all(),
      bump_mode_use: ThmBumpMode::Use.into(),
      minimum_gloss_power: GenerateBumpResult::MINIMUM_GLOSS_POWER,
    }
  }
}
