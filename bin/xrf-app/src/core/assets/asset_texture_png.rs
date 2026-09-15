use xrf_dds::DdsFile;
use xrf_error::XrfResult;
use xrf_vfs::XrayProbe;

use crate::core::assets::asset_read::read_located_asset;

/// Decode a located texture into the PNG bytes a webview can display.
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
