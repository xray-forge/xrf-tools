use tauri::State;
use xrf_error::XrfError;
use xrf_vfs::{XrayAsset, XrayRoots};

use crate::core::assets::{AssetMountState, AssetTextureDescriptor};
use crate::core::types::TauriResult;

/// Report the shape of a texture, without decoding it into a picture.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_texture"))]
#[tauri::command(rename = "describe_texture")]
pub async fn archives_describe_texture(
  roots: XrayRoots,
  logical_path: String,
  assets: State<'_, AssetMountState>,
) -> TauriResult<AssetTextureDescriptor> {
  log::info!("Describing image: {logical_path}");

  assets
    .with_probe(&roots, |probe| {
      let asset: XrayAsset = probe.find(&logical_path)?.get_asset().cloned().ok_or_else(|| {
        XrfError::new_asset_error(format!("'{logical_path}' resolves to nothing in the mounted roots"))
      })?;

      // A header that will not parse costs the shape, not the descriptor - the same best-effort the sound path takes.
      AssetTextureDescriptor::describe(probe, &asset)
        .ok_or_else(|| XrfError::new_asset_error(format!("'{logical_path}' could not be read at all")))
    })?
    .map_err(|error| format!("Failed to describe image '{logical_path}': {error}"))
}
