use serde::Serialize;
use tauri::State;
use xrf_error::XrfError;
use xrf_extension::XrayExtensionOf;
use xrf_texture::ImageShape;
use xrf_vfs::XrayRoots;

use crate::core::assets::{AssetMountState, read_located_asset};
use crate::core::types::TauriResult;

/// What a picture the webview renders itself turns out to be.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageDescriptor {
  /// Absent when the bytes carry no header this reads, which is a picture worth neither drawing nor measuring.
  pub shape: Option<ImageShape>,
  /// What to hand the bytes over as, taken from the extension rather than from the content.
  pub media_type: String,
}

/// Report the shape of a picture the webview draws as it stands.
///
/// # Errors
///
/// Returns an error when the path resolves to nothing, its bytes cannot be read, or its extension is not one a
/// webview draws - which the caller decided before asking, so reaching it means the two disagree.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_image"))]
#[tauri::command(rename = "describe_image")]
pub async fn archives_describe_image(
  roots: XrayRoots,
  logical_path: String,
  assets: State<'_, AssetMountState>,
) -> TauriResult<ImageDescriptor> {
  log::info!("Describing image: {logical_path}");

  let media_type: &'static str = XrayExtensionOf::of(&logical_path)
    .known()
    .and_then(|extension| extension.get_media_type())
    .ok_or_else(|| format!("Failed to describe image '{logical_path}': no webview renders this extension"))?;

  let bytes: Vec<u8> = assets
    .with_probe(&roots, |probe| read_located_asset(probe, &logical_path))?
    .map_err(|error: XrfError| format!("Failed to describe image '{logical_path}': {error}"))?;

  // A picture whose header will not parse is still worth handing to the webview: it may know a variant this does
  // not, and refusing the shape would take the drawable bytes down with it.
  let shape: Option<ImageShape> = ImageShape::of_bytes(&bytes);

  if shape.is_none() {
    log::warn!("Image '{logical_path}' carries no header this reads");
  }

  Ok(ImageDescriptor {
    shape,
    media_type: media_type.to_owned(),
  })
}
