use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::SessionId;
use xrf_dds::{DdsMipFilter, DdsMipmaps};
use xrf_texture::TextureEncodingQuality;
use xrf_utils::{error_to_string, format_path};
use xrf_vfs::{XrayAsset, XrayAssetType, XrayRoots, XraySearch};

use crate::describe::{TextureFiles, TextureSource};
use crate::edit::TextureEditTargets;

/// What a format comparison was asked to weigh.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TexturesCompareRequest {
  pub session_id: SessionId,
  /// The texture to re-encode, named the way `describe` names one.
  pub source: TextureSource,
  pub roots: XrayRoots,
  /// Kernel the chain is reduced with, by its SDK name, or `None` to weigh the base level alone.
  pub mip_filter: Option<String>,
  pub quality: TextureEncodingQuality,
}

impl TexturesCompareRequest {
  /// The chain every candidate is weighed over.
  ///
  /// # Errors
  ///
  /// Returns an error naming the kernel when it is not one of the fourteen.
  pub fn to_mipmaps(&self) -> TauriResult<DdsMipmaps> {
    match self.mip_filter.as_deref() {
      None => Ok(DdsMipmaps::Disabled),
      Some(label) => DdsMipFilter::from_label(label)
        .map(DdsMipmaps::Filtered)
        .ok_or_else(|| format!("Unexpected mip filter '{label}'")),
    }
  }

  /// The stored bytes of the `.dds` the source names, with the two files on disk its encodes may be saved over, as
  /// `describe` locates them; none for a texture served out of an archive.
  ///
  /// # Errors
  ///
  /// Returns an error when the roots cannot be mounted, the reference resolves to nothing, or the bytes cannot be read.
  pub fn read_dds(&self, assets: &AssetMountState) -> TauriResult<(Vec<u8>, Option<TextureFiles>)> {
    match &self.source {
      TextureSource::File { .. } => {
        // The texture beside what was named rather than the named file itself: a person opens a texture by picking
        // either half of the pair, and weighing the descriptor's own bytes reads a chunked file as a dds.
        let path: PathBuf = self
          .source
          .to_texture_path()
          .ok_or_else(|| String::from("A texture outside every root has to be named by a file path"))?;
        let bytes: Vec<u8> =
          std::fs::read(&path).map_err(|error| format!("Cannot read '{}': {error}", format_path(&path)))?;

        Ok((bytes, Some(TextureFiles::of(&path))))
      }
      TextureSource::Asset { reference } => {
        let search: XraySearch = assets.search(&self.roots)?;
        let texture: XrayAsset = search
          .resolve(XrayAssetType::Dds, reference)
          .map_err(|error| format!("Rejected texture '{reference}': {error}"))?
          .get_asset()
          .cloned()
          .ok_or_else(|| format!("Texture '{reference}' resolves to no file"))?;
        let descriptor: Option<XrayAsset> = search
          .resolve(XrayAssetType::Thm, reference)
          .ok()
          .and_then(|resolution| resolution.get_asset().cloned());

        Ok((
          search.read_asset(&texture).map_err(error_to_string)?,
          TextureEditTargets::locate(&texture, descriptor.as_ref()),
        ))
      }
    }
  }
}
