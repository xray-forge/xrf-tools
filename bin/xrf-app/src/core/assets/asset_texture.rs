use serde::Serialize;
use xrf_dds::{DdsFile, DdsMetadata};
use xrf_error::XrfResult;
use xrf_vfs::{XrayAsset, XrayProbe};

use crate::core::assets::asset_read::read_located_asset;

/// What a texture file is, once it has been located.
///
/// Reported by the command that resolved the reference rather than derived by the frontend: the facts all come from a
/// DDS header, `xrf-dds` already reads one, and a renderer-side reimplementation would name the same formats
/// differently than the `verify-ogf` census does.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetTextureDescriptor {
  /// Bytes the file occupies, which is also what a renderer uploads for a block-compressed texture.
  pub size: u64,
  /// Header facts, absent when the bytes are not a readable DDS.
  ///
  /// Nested rather than four independent options, so a partially known shape cannot be described: either the header
  /// parsed and every field is from it, or it did not and the size is all that is known.
  pub shape: Option<AssetTextureShape>,
}

/// Pixel layout a DDS header declares.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetTextureShape {
  pub width: u32,
  pub height: u32,
  /// Levels the file carries, one meaning no mip chain at all.
  ///
  /// Load bearing rather than trivia: a texture without mips has to be sampled with a linear filter or webgl renders it
  /// black, and 1,805 of Anomaly's 2,197 distinct textures ship without one.
  pub mipmap_levels: u32,
  /// Format name from [`DdsMetadata::get_format_label`], so the viewer and the sweep agree on what a file is.
  pub format: String,
}

impl AssetTextureDescriptor {
  /// Describes a located texture, or nothing when its bytes cannot be reached at all.
  ///
  /// A loose file is read by path, which costs only the header. An archived entry has no path to read a prefix of, so
  /// its bytes come out of the volume whole - the same read the frontend makes a moment later to upload it, which is
  /// the price of describing an archived texture at all.
  ///
  /// A header that will not parse costs the shape, not the descriptor: the byte count is still a fact, and the files
  /// whose headers this reader refuses are exactly the ones worth knowing the size of.
  pub fn describe(probe: &XrayProbe, asset: &XrayAsset) -> Option<Self> {
    match asset.to_physical_path() {
      Some(path) => {
        let metadata: Option<DdsMetadata> = DdsFile::read_metadata_from_path(&path).ok();
        let size: u64 = match &metadata {
          Some(metadata) => metadata.file_size,
          None => std::fs::metadata(&path).ok()?.len(),
        };

        Some(Self {
          size,
          shape: metadata.as_ref().map(AssetTextureShape::from_metadata),
        })
      }
      None => {
        let bytes: Vec<u8> = probe.read_asset_bytes(asset).ok()?;

        Some(Self {
          size: bytes.len() as u64,
          shape: DdsFile::read_metadata_from_bytes(&bytes)
            .ok()
            .as_ref()
            .map(AssetTextureShape::from_metadata),
        })
      }
    }
  }
}

/// Decode a located texture into the PNG bytes a webview can display.
///
/// The one transcode in the application, because the answer has to be the same wherever it is asked: the archives
/// preview shows a DDS this way, and the model viewer falls back to it for the layouts its own loader refuses.
///
/// PNG rather than raw pixels because the webview decodes it natively and the payload stays a fraction of the size,
/// which matters for the 2048 square terrain textures this is reached for.
///
/// # Errors
///
/// Returns an error when the asset cannot be read, or when its layout is one [`DdsFile::decode_rgba`] does not decode:
/// `A8` alpha-only, `R5G6B5`, 16bpp alpha-luminance, `X8R8G8B8` and `L8`, which is 305 of the 28,606 files measured
/// across the reference trees. That table is the one place the list lives.
pub fn read_texture_png(probe: &XrayProbe, logical_path: &str) -> XrfResult<Vec<u8>> {
  let bytes: Vec<u8> = read_located_asset(probe, logical_path)?;

  Ok(DdsFile::read_from_bytes(&bytes).and_then(|dds| dds.to_png())?.bytes)
}

impl AssetTextureShape {
  fn from_metadata(metadata: &DdsMetadata) -> Self {
    Self {
      width: metadata.width,
      height: metadata.height,
      mipmap_levels: metadata.mipmap_levels,
      format: metadata.get_format_label(),
    }
  }
}
