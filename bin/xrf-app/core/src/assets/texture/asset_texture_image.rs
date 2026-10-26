use xrf_dds::{DdsFile, RgbaImage};
use xrf_error::XrfResult;

/// A texture file's pixels in the two shapes the page reads them in: a picture to show, and texels to measure.
pub struct AssetTextureImage;

impl AssetTextureImage {
  /// Decodes a texture file into the PNG bytes a webview can display.
  ///
  /// PNG rather than raw pixels because the webview decodes it natively and the payload stays a fraction of the size,
  /// which matters for the 2048 square terrain textures this is reached for.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are no texture, or when its layout is one [`DdsFile::decode_rgba`] does not
  /// decode: `A8` alpha-only, `R5G6B5`, 16bpp alpha-luminance, `X8R8G8B8` and `L8`, which is 305 of the 28,606 files
  /// measured across the reference trees. That table is the one place the list lives.
  pub fn encode_png(bytes: &[u8]) -> XrfResult<Vec<u8>> {
    Ok(DdsFile::read_from_bytes(bytes).and_then(|dds| dds.to_png())?.bytes)
  }

  /// A texture file's top level as plain texels, whatever its layout: its width and height as little-endian `u32`s,
  /// then four bytes a texel, row major, exactly as decoded - never a picture a webview would premultiply or convert.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are no texture, or its layout is one that does not decode.
  pub fn decode_texels(bytes: &[u8]) -> XrfResult<Vec<u8>> {
    let image: RgbaImage = DdsFile::read_from_bytes(bytes)?.decode_rgba(0)?;
    let mut texels: Vec<u8> = Vec::with_capacity(8 + image.as_raw().len());

    texels.extend_from_slice(&image.width().to_le_bytes());
    texels.extend_from_slice(&image.height().to_le_bytes());
    texels.extend_from_slice(image.as_raw());

    Ok(texels)
  }
}
