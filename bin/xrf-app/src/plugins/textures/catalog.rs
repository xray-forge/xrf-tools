use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use xrf_material::{XrayBumpNaming, XrayMaterialResolver};
use xrf_vfs::{XrayAsset, XrayAssetType, XrayProbe, XrayRoots};

use crate::plugins::textures::source::TextureSource;

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

/// How a listing addressed what it found.
///
/// Two shapes rather than one because the two cases want opposite defaults. In a game tree the files that yield no
/// engine reference are a level's lightmaps, and burying two thousand named textures in them is the bug; in a folder
/// somebody is authoring in, those files are the entire point and there are no references to be had at all.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TextureCatalogMode {
  /// The game tree: listed by engine reference, archives included, files outside `textures\` counted not listed.
  #[default]
  Roots,
  /// A plain directory: every `.dds` under it, addressed by its own path.
  LooseDirectory,
}

/// One texture name and the files the roots hold for it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureEntry {
  /// What to call this row: an engine reference such as `ston\ston_beton05`, or a loose file's path below its root.
  ///
  /// Unique within one listing either way, so a tree can key on it. It is a label rather than an address; what to
  /// open is `source`, because a loose file has no reference to be resolved back into.
  pub reference: String,
  /// How to open this row, which is the address the describe and every write take.
  pub source: TextureSource,
  pub role: TextureRole,
  /// The `.dds` the roots answer for this reference, winner first.
  pub texture: Option<XrayAsset>,
  /// The `.thm` the roots answer for this reference, winner first.
  pub descriptor: Option<XrayAsset>,
}

impl TextureEntry {
  /// A row of the game tree, opened by the reference it is listed under.
  fn of_reference(reference: String) -> Self {
    Self {
      role: TextureRole::of_reference(&reference),
      source: TextureSource::Asset {
        reference: reference.clone(),
      },
      reference,
      texture: None,
      descriptor: None,
    }
  }

  /// A row of a loose directory, opened by the file itself.
  fn of_file(label: String, path: PathBuf) -> Self {
    Self {
      role: TextureRole::of_reference(&label),
      source: TextureSource::File {
        path: path.to_string_lossy().into_owned(),
      },
      reference: label,
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
  /// How this listing was made, which decides what its rows are addressed by and whether a sweep can badge them.
  pub mode: TextureCatalogMode,
  /// The roots the catalog was listed from, so a later read searches what the listing searched.
  pub roots: XrayRoots,
  /// A `textures.ltx` the roots hold, or `None`. Its declarations are not read, so a surface says where it matters.
  pub textures_ltx: Option<XrayAsset>,
  pub entries: Vec<TextureEntry>,
  /// `.dds` files the roots hold outside `textures\`, which no engine reference names and this catalog leaves out.
  ///
  /// Counted rather than dropped silently: a level's lightmaps are the usual case, and a person wondering where a file
  /// went deserves the number.
  pub outside_textures_count: u32,
}

impl TextureCatalog {
  /// Lists what a probe reaches, the way this mode addresses it.
  pub fn list(probe: &XrayProbe, roots: XrayRoots, mode: TextureCatalogMode) -> Self {
    match mode {
      TextureCatalogMode::Roots => Self::list_roots(probe, roots),
      TextureCatalogMode::LooseDirectory => Self::list_loose(probe, roots),
    }
  }

  /// Lists the textures and descriptors a probe reaches, folded onto references.
  fn list_roots(probe: &XrayProbe, roots: XrayRoots) -> Self {
    let mut entries: BTreeMap<String, TextureEntry> = BTreeMap::new();
    let mut outside_textures_count: u32 = 0;

    for asset in probe.list_assets_of_type(XrayAssetType::Dds) {
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

    for asset in probe.list_assets_of_type(XrayAssetType::Thm) {
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
      textures_ltx: XrayMaterialResolver::find_textures_ltx(probe),
      entries: entries.into_values().collect(),
      outside_textures_count,
    }
  }

  /// Lists every `.dds` of a plain directory, addressed by its own path.
  ///
  /// Keyed by the logical path without its extension rather than by the file stem alone: the stem is what a row is
  /// labelled by, but two folders under one root may each hold a `wall01.dds`, and a listing that folded them together
  /// would lose one of them. Nothing is counted as outside, because in this mode there is no inside.
  fn list_loose(probe: &XrayProbe, roots: XrayRoots) -> Self {
    let mut entries: BTreeMap<String, TextureEntry> = BTreeMap::new();

    for asset in probe.list_assets_of_type(XrayAssetType::Dds) {
      if let (Some(key), Some(path)) = (to_loose_key(&asset), asset.to_physical_path()) {
        entries
          .entry(key.clone())
          .or_insert_with(|| TextureEntry::of_file(key, path))
          .texture = Some(asset);
      }
    }

    for asset in probe.list_assets_of_type(XrayAssetType::Thm) {
      if let (Some(key), Some(path)) = (to_loose_key(&asset), asset.to_physical_path()) {
        entries
          .entry(key.clone())
          .or_insert_with(|| TextureEntry::of_file(key, path))
          .descriptor = Some(asset);
      }
    }

    Self {
      mode: TextureCatalogMode::LooseDirectory,
      roots,
      textures_ltx: None,
      entries: entries.into_values().collect(),
      outside_textures_count: 0,
    }
  }
}

/// What a loose row is keyed and labelled by: its path below the root, without the extension.
fn to_loose_key(asset: &XrayAsset) -> Option<String> {
  let path: &str = asset.get_logical_path().as_str();
  let extension: &str = asset.get_asset_type()?.get_rules()?.extension;

  path.strip_suffix(extension).map(str::to_owned)
}
