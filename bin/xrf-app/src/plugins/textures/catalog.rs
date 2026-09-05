use std::collections::BTreeMap;

use serde::Serialize;
use xrf_material::{XrayBumpNaming, XrayMaterialResolver};
use xrf_vfs::{XrayAsset, XrayAssetType, XrayProbe, XrayRoots};

/// What a texture name is by convention, before any descriptor has been read.
///
/// Read off the name so a tree can fold a pair under its texture the moment the listing arrives; which pairs are
/// declared, and by whom, is what the sweep then says. The convention itself is `xrf-material`'s, shared with the
/// renderer's fallback rule and the companion derivation.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TextureRole {
  /// A texture a mesh or a level binds by name.
  Texture,
  /// The first half of a bump pair: packed normal and gloss.
  Bump,
  /// The second half of a bump pair: packed error and height.
  BumpCompanion,
}

impl TextureRole {
  pub fn of_reference(reference: &str) -> Self {
    if XrayBumpNaming::is_companion(reference) {
      Self::BumpCompanion
    } else if XrayBumpNaming::is_conventional_bump(reference) {
      Self::Bump
    } else {
      Self::Texture
    }
  }
}

/// One texture name and the files the roots hold for it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureEntry {
  /// The engine reference, such as `ston\ston_beton05`.
  pub reference: String,
  pub role: TextureRole,
  /// The `.dds` the roots answer for this reference, winner first.
  pub texture: Option<XrayAsset>,
  /// The `.thm` the roots answer for this reference, winner first.
  pub descriptor: Option<XrayAsset>,
}

impl TextureEntry {
  fn new(reference: String) -> Self {
    Self {
      role: TextureRole::of_reference(&reference),
      reference,
      texture: None,
      descriptor: None,
    }
  }
}

/// Every texture the roots hold, once per reference, in reference order.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureCatalog {
  /// The roots the catalog was listed from, so a later read searches what the listing searched.
  pub roots: XrayRoots,
  /// A `textures.ltx` the roots hold, or `None`. Its declarations are not read, so a surface says so where it matters.
  pub textures_ltx: Option<XrayAsset>,
  pub entries: Vec<TextureEntry>,
  /// `.dds` files the roots hold outside `textures\`, which no engine reference names and this catalog leaves out.
  ///
  /// Counted rather than dropped silently: a level's lightmaps are the usual case, and a person wondering where a file
  /// went deserves the number.
  pub outside_textures_count: u32,
}

impl TextureCatalog {
  /// Lists the textures and descriptors a probe reaches, folded onto references.
  pub fn list(probe: &XrayProbe, roots: XrayRoots) -> Self {
    let mut entries: BTreeMap<String, TextureEntry> = BTreeMap::new();
    let mut outside_textures_count: u32 = 0;

    for asset in probe.list_assets_of_type(XrayAssetType::Dds) {
      match asset.to_reference() {
        Some(reference) => {
          entries
            .entry(reference.clone())
            .or_insert_with(|| TextureEntry::new(reference))
            .texture = Some(asset);
        }
        None => outside_textures_count += 1,
      }
    }

    for asset in probe.list_assets_of_type(XrayAssetType::Thm) {
      if let Some(reference) = asset.to_reference() {
        entries
          .entry(reference.clone())
          .or_insert_with(|| TextureEntry::new(reference))
          .descriptor = Some(asset);
      }
    }

    Self {
      roots,
      textures_ltx: XrayMaterialResolver::find_textures_ltx(probe),
      entries: entries.into_values().collect(),
      outside_textures_count,
    }
  }
}
