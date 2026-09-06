//! The names the SDK gives the numbers a descriptor stores.
//!
//! Built from `xrf-db`'s own tables rather than transcribed into TypeScript. Every one of these enums serializes as the
//! `u32` it is stored as, with an `Unknown(u32)` variant for a value the SDK never named, so a surface offering them
//! has to get a list of numbers from somewhere. Getting it from here means a value gaining a name in `xrf-db` gains it
//! in the editor with no second table to remember.

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
///
/// Answered once when the editor opens rather than carried on every description: it is the same table for every
/// texture in every root, and a description that repeated it would spend it thousands of times over a sweep.
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
  ///
  /// Named rather than left to a surface to recognise, because a tool that has just written a pair has to point the
  /// descriptor at it and there is exactly one value that does. Matching on the display label would work until
  /// somebody rewords it; matching on the number would work until it is spelled differently in two places.
  pub bump_mode_use: u32,
}

impl TextureVocabulary {
  /// The whole table, in the order each enum's own `NAMED` lists it.
  ///
  /// Order is the SDK's rather than alphabetical, because these are combo boxes in the editor it is modelled on and a
  /// person who knows that dialog knows where its entries sit.
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
///
/// The label arrives as a function rather than through a trait these types implement, because every one of them
/// already has a `label` and the only thing a trait would add is a second name for it. Passing it here also puts the
/// spelling at the call site, so a reader of `describe` can see which of the five is which without leaving the line.
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
