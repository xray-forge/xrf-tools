use tauri::State;
use xrf_error::XrfError;
use xrf_vfs::XrayAsset;

use crate::core::assets::{AssetTextureDescriptor, AssetWorldSpec, AssetWorldState};
use crate::core::types::TauriResult;

/// Report the shape of a texture, without decoding it into a picture.
///
/// Paired with `archives|read_image`, which serves the PNG the webview displays. Both are addressed by the same world
/// and logical path, so the dimensions on screen belong to the picture beside them.
///
/// Answers with the source DDS facts rather than the PNG's: format and mip count survive the description and would not
/// survive the transcode, and a viewer of X-Ray textures wants both.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_image"))]
#[tauri::command(rename = "describe_image")]
pub async fn archives_describe_image(
  world: AssetWorldSpec,
  logical_path: String,
  assets: State<'_, AssetWorldState>,
) -> TauriResult<AssetTextureDescriptor> {
  log::info!("Describing image: {logical_path}");

  assets
    .with_probe(&world, |probe| {
      let asset: XrayAsset = probe.find(&logical_path)?.get_asset().cloned().ok_or_else(|| {
        XrfError::new_asset_error(format!("'{logical_path}' resolves to nothing in the mounted world"))
      })?;

      // A header that will not parse costs the shape, not the descriptor - the same best-effort the sound path takes.
      AssetTextureDescriptor::describe(probe, &asset)
        .ok_or_else(|| XrfError::new_asset_error(format!("'{logical_path}' could not be read at all")))
    })?
    .map_err(|error| format!("Failed to describe image '{logical_path}': {error}"))
}
