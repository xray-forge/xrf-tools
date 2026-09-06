//! What each of the editor's four commands was asked to do.
//!
//! One module for the whole domain, as every plugin over the three-argument limit has: a reader asking what the
//! textures editor accepts over IPC reads one file rather than four command bodies.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use xrf_dds::{DdsMipFilter, DdsMipmaps};
use xrf_vfs::XrayRoots;

use crate::core::types::TauriResult;
use crate::plugins::textures::descriptor_form::TextureDescriptorForm;
use crate::plugins::textures::encoding::{TextureEncodingFormat, TextureEncodingQuality};
use crate::plugins::textures::file_stamp::TextureFileStamp;
use crate::plugins::textures::lease::to_texture_lease_key;
use crate::plugins::textures::source::TextureSource;

/// One file a write addresses, and what was there when the editor read it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureSaveTarget {
  /// Absolute path of the file to replace or create.
  pub path: String,
  /// The stamp the editor read there, or `None` for a file it is creating.
  pub expected: Option<TextureFileStamp>,
}

/// The descriptor half of a save.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureDescriptorSave {
  pub target: TextureSaveTarget,
  pub form: TextureDescriptorForm,
}

/// The texture half of a save, which is one of the candidates a comparison already encoded.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureEncodingSave {
  pub target: TextureSaveTarget,
  pub format: TextureEncodingFormat,
}

/// What one node's save was asked to write.
///
/// Both halves are optional and independent: a node may be dirty in its descriptor, in its pixels, or in both, and a
/// save that could only do the pair would make the common case - a flag changed on a texture nobody re-encoded -
/// impossible to express.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TexturesSaveRequest {
  pub descriptor: Option<TextureDescriptorSave>,
  pub texture: Option<TextureEncodingSave>,
}

/// What a texture rebuild was asked to do.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TexturesBuildRequest {
  /// Path of the `.dds` to write, which is the file beside the descriptor.
  pub destination: String,
  /// Path of the image to encode, of whatever kind `image` decodes.
  pub source: String,
  /// The descriptor to read as a recipe, as the editor currently has it rather than as it is on disk.
  ///
  /// The form rather than the file, because a build should produce what the editor's own format and flags describe. A
  /// person who has changed the format and not saved yet wants to see that format built.
  pub descriptor: TextureDescriptorForm,
  pub quality: TextureEncodingQuality,
}

/// What a bump pair generation was asked to do.
///
/// The height source is required and everything else refines it, exactly as the SDK's generator has it: normals are
/// derived from the height alone and gloss is a separate plane, so a caller with only a height map still gets a pair.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TexturesMakeBumpRequest {
  /// Path of the texture the pair belongs to, without the `_bump` suffix or an extension.
  pub destination: String,
  /// Path of the image the relief is read from, averaged across its colour channels.
  pub height: String,
  /// Path of a gloss mask, averaged the same way.
  ///
  /// When absent the whole surface takes [`Self::gloss_constant`], which is what a texture authored without a mask
  /// needs and what the SDK's own dialog offers.
  pub gloss: Option<String>,
  pub gloss_constant: f32,
  /// Path of a normal map to use instead of deriving one from the height, of the same size.
  pub normal_map: Option<String>,
  /// `bump_virtual_height` of the descriptor, read here and nowhere at runtime.
  pub virtual_height: f32,
  /// Kernel the pair's chain is reduced with, by its SDK name.
  ///
  /// Defaults to `Box` at the caller, because that is what the SDK's generator leaves it at: `DXTCompressBump` builds
  /// its `STextureParams` and overrides only the flags, the type and the format.
  pub mip_filter: Option<String>,
  pub quality: TextureEncodingQuality,
}

/// What a format comparison was asked to weigh.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TexturesCompareRequest {
  /// The texture to re-encode, named the way `describe` names one.
  ///
  /// A source rather than an engine reference, because a file outside every tree has no reference and is addressed by
  /// its path. The label a surface shows still comes from the description; this is the address.
  pub source: TextureSource,
  pub roots: XrayRoots,
  /// Kernel the chain is reduced with, by its SDK name, or `None` to weigh the base level alone.
  ///
  /// Not read from the descriptor. A comparison answers "what would this texture cost in each format", and the answer
  /// has to be about one chain built one way, or the figures are not comparable with each other.
  pub mip_filter: Option<String>,
  pub quality: TextureEncodingQuality,
}

impl TexturesSaveRequest {
  /// Every file this save would write, as lease keys.
  ///
  /// Taken before the work starts, so a build or a generation aimed at the same texture is refused at registration
  /// rather than discovered halfway through a rename.
  pub fn to_lease_keys(&self) -> Vec<String> {
    [
      self.descriptor.as_ref().map(|save| &save.target.path),
      self.texture.as_ref().map(|save| &save.target.path),
    ]
    .into_iter()
    .flatten()
    .map(|path| to_texture_lease_key(Path::new(path)))
    .collect()
  }
}

impl TexturesBuildRequest {
  /// The file this build would write, as a lease key.
  pub fn to_lease_keys(&self) -> Vec<String> {
    vec![to_texture_lease_key(Path::new(&self.destination))]
  }

  pub fn to_destination(&self) -> PathBuf {
    PathBuf::from(&self.destination)
  }
}

impl TexturesMakeBumpRequest {
  /// The chain the pair is written with.
  ///
  /// A pair always carries one: the SDK's generator sets `flGenerateMipMaps` and never clears it, so a request naming
  /// no kernel gets the `Box` the generator leaves its parameters at rather than a flat texture.
  ///
  /// # Errors
  ///
  /// Returns an error naming the kernel when it is not one of the fourteen.
  pub fn to_mip_filter(&self) -> TauriResult<DdsMipFilter> {
    match to_mipmaps(self.mip_filter.as_deref())? {
      DdsMipmaps::Filtered(filter) => Ok(filter),
      DdsMipmaps::Disabled => Ok(DdsMipFilter::Box),
    }
  }
}

impl TexturesCompareRequest {
  /// The chain every candidate is weighed over.
  ///
  /// # Errors
  ///
  /// Returns an error naming the kernel when it is not one of the fourteen.
  pub fn to_mipmaps(&self) -> TauriResult<DdsMipmaps> {
    to_mipmaps(self.mip_filter.as_deref())
  }
}

/// The chain a caller asked for, by the kernel's SDK name.
///
/// Named rather than numbered because the name is what the SDK's own filter combo shows and what `dds convert` takes,
/// so a filter chosen in the editor and one typed at the CLI are the same token.
fn to_mipmaps(mip_filter: Option<&str>) -> TauriResult<DdsMipmaps> {
  match mip_filter {
    None => Ok(DdsMipmaps::Disabled),
    Some(label) => DdsMipFilter::from_label(label)
      .map(DdsMipmaps::Filtered)
      .ok_or_else(|| format!("Unexpected mip filter '{label}'")),
  }
}
