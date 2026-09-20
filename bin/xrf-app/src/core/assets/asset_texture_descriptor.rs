use std::path::Path;

use serde::Serialize;
use xrf_dds::{DdsFile, DdsMetadata};
use xrf_vfs::{XrayAsset, XrayProbe};

use crate::core::assets::asset_texture_shape::AssetTextureShape;

/// What a texture file is, once it has been located.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetTextureDescriptor {
  /// Bytes the file occupies, which is also what a renderer uploads for a block-compressed texture.
  pub size: u64,
  /// Header facts, absent when the bytes are not a readable DDS.
  pub shape: Option<AssetTextureShape>,
}

impl AssetTextureDescriptor {
  /// Describes a located texture, or nothing when its bytes cannot be reached at all.
  ///
  /// A loose file is read by path, which costs only the header. An archived entry has no path to read a prefix of, so
  /// its bytes come out of the volume whole - the same read the frontend makes a moment later to upload it, which is
  /// the price of describing an archived texture at all.
  pub fn describe(probe: &XrayProbe, asset: &XrayAsset) -> Option<Self> {
    match asset.to_physical_path() {
      Some(path) => Self::describe_path(&path),
      None => {
        let bytes: Vec<u8> = probe.read_asset_bytes(asset).ok()?;

        Some(Self {
          size: bytes.len() as u64,
          shape: AssetTextureShape::of_bytes(&bytes),
        })
      }
    }
  }

  /// Describes a texture file by path, for one no mount holds.
  pub fn describe_path(path: &Path) -> Option<Self> {
    let metadata: Option<DdsMetadata> = DdsFile::read_metadata_from_path(path).ok();

    Some(Self {
      size: match &metadata {
        Some(metadata) => metadata.file_size,
        None => std::fs::metadata(path).ok()?.len(),
      },
      shape: metadata.as_ref().map(AssetTextureShape::from),
    })
  }
}
