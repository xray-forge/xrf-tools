use std::collections::BTreeMap;

use serde::Serialize;
use xrf_material::XrayMaterialResolver;
use xrf_vfs::{XrayAsset, XrayAssetType, XrayRoots, XraySearch};

use crate::browse::{TextureCatalogMode, TextureEntry};

/// Every texture the roots hold, once per reference, in reference order.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureCatalog {
  /// How this listing was made, which decides what its rows are addressed by and whether a sweep can badge them.
  pub mode: TextureCatalogMode,
  /// The roots the catalog was listed from, so a later read searches what the listing searched.
  pub roots: XrayRoots,
  /// A `textures.ltx` the roots hold, or `None`. Its declarations are not read, so a surface says where it matters.
  pub textures_ltx: Option<XrayAsset>,
  pub entries: Vec<TextureEntry>,
  /// `.dds` files the roots hold outside `textures\`, which no engine reference names and this catalog leaves out.
  pub outside_textures_count: u32,
  /// Loose files whose path is not Unicode, which the page could not name back and this catalog leaves out.
  pub unnamed_count: u32,
}

impl TextureCatalog {
  /// Lists what a search reaches, the way this mode addresses it.
  pub fn list(search: &XraySearch, roots: XrayRoots, mode: TextureCatalogMode) -> Self {
    match mode {
      TextureCatalogMode::Roots => Self::list_roots(search, roots),
      TextureCatalogMode::LooseDirectory => Self::list_loose(search, roots),
    }
  }

  /// Lists the textures and descriptors a search reaches, folded onto references.
  fn list_roots(search: &XraySearch, roots: XrayRoots) -> Self {
    let mut entries: BTreeMap<String, TextureEntry> = BTreeMap::new();
    let mut outside_textures_count: u32 = 0;

    for asset in search.list_assets(XrayAssetType::Dds) {
      match asset.to_reference() {
        Some(reference) => {
          entries
            .entry(reference.clone())
            .or_insert_with(|| TextureEntry::of_reference(reference))
            .texture = Some(asset);
        }
        None => outside_textures_count += 1,
      }
    }

    for asset in search.list_assets(XrayAssetType::Thm) {
      if let Some(reference) = asset.to_reference() {
        entries
          .entry(reference.clone())
          .or_insert_with(|| TextureEntry::of_reference(reference))
          .descriptor = Some(asset);
      }
    }

    Self {
      mode: TextureCatalogMode::Roots,
      roots,
      textures_ltx: XrayMaterialResolver::find_textures_ltx(search),
      entries: entries.into_values().collect(),
      outside_textures_count,
      unnamed_count: 0,
    }
  }

  /// Lists every `.dds` of a plain directory, addressed by its own path; a file whose path is not Unicode is counted,
  /// since a path the page cannot hold would name another file or none when sent back.
  fn list_loose(search: &XraySearch, roots: XrayRoots) -> Self {
    let mut entries: BTreeMap<String, TextureEntry> = BTreeMap::new();
    let mut unnamed_count: u32 = 0;

    for (asset_type, is_texture) in [(XrayAssetType::Dds, true), (XrayAssetType::Thm, false)] {
      for asset in search.list_assets(asset_type) {
        let (Some(key), Some(physical)) = (TextureEntry::loose_key(&asset), asset.to_physical_path()) else {
          continue;
        };
        let Some(path) = physical.to_str().map(str::to_owned) else {
          unnamed_count += 1;

          continue;
        };
        let entry: &mut TextureEntry = entries
          .entry(key.clone())
          .or_insert_with(|| TextureEntry::of_file(key, path));

        if is_texture {
          entry.texture = Some(asset);
        } else {
          entry.descriptor = Some(asset);
        }
      }
    }

    Self {
      mode: TextureCatalogMode::LooseDirectory,
      roots,
      textures_ltx: None,
      entries: entries.into_values().collect(),
      outside_textures_count: 0,
      unnamed_count,
    }
  }
}
