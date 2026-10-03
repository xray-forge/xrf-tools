use serde::Deserialize;
use tauri::{AppHandle, Manager};
use xrf_dds::{DdsFile, RgbaImage};
use xrf_vfs::XrayRoots;

use crate::core::assets::{AssetMountState, read_located_asset};
use crate::core::execution::ExecutionState;
use crate::core::transport::TransportAnswer;
use crate::core::types::TauriResult;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TexturesReadTexelsRequest {
  roots: XrayRoots,
  logical_path: String,
}

/// A texture's top level as plain texels, whatever its layout: its width and height as little-endian `u32`s, then four
/// bytes a texel, row major, exactly as decoded - never a picture a webview would premultiply or convert.
pub(crate) async fn textures_read_texels(
  app: AppHandle,
  request: TexturesReadTexelsRequest,
) -> TauriResult<TransportAnswer> {
  let TexturesReadTexelsRequest { roots, logical_path } = request;

  log::info!("Reading texels: {logical_path}");

  let assets: AssetMountState = AssetMountState::clone(&app.state::<AssetMountState>());
  let texels: Vec<u8> = app
    .state::<ExecutionState>()
    .run_blocking("Reading texels", move || {
      assets
        .with_probe(&roots, |probe| {
          let image: RgbaImage =
            DdsFile::read_from_bytes(&read_located_asset(probe, &logical_path)?)?.decode_rgba(0)?;
          let mut texels: Vec<u8> = Vec::with_capacity(8 + image.as_raw().len());

          texels.extend_from_slice(&image.width().to_le_bytes());
          texels.extend_from_slice(&image.height().to_le_bytes());
          texels.extend_from_slice(image.as_raw());

          xrf_error::XrfResult::Ok(texels)
        })?
        .map_err(|error| format!("Failed to read the texels of '{logical_path}': {error}"))
    })
    .await??;

  Ok(TransportAnswer::octets(texels))
}
