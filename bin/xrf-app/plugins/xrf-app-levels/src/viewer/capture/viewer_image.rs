use std::fs;
use std::io::Cursor;
use std::path::Path;

use image::codecs::tga::TgaEncoder;
use image::{ExtendedColorType, ImageEncoder, ImageFormat, ImageReader, Rgba, RgbaImage};
use xrf_app_core::TauriResult;
use xrf_dds::DDS_BLOCK_SIZE;

/// The picture vocabulary both exports share: reading a capture back, sizing and flattening it, and encoding it.
pub struct ViewerImage;

impl ViewerImage {
  /// An export's pixels: the longer side `long_side` at most the capture's, a DDS rounded to whole blocks.
  ///
  /// The rounding resamples rather than pads: the game stretches the texture over the map's area, so it must cover it.
  pub fn get_export_size((width, height): (u32, u32), long_side: u32, is_blocked: bool) -> (u32, u32) {
    let longest: u32 = width.max(height).max(1);
    let scale: f64 = f64::from(long_side.clamp(1, longest)) / f64::from(longest);
    let side = |pixels: u32| -> u32 {
      let scaled: f64 = (f64::from(pixels) * scale).round().max(1.0);

      if is_blocked {
        let block: f64 = f64::from(DDS_BLOCK_SIZE);

        ((scaled / block).round().max(1.0) * block) as u32
      } else {
        scaled as u32
      }
    };

    (side(width), side(height))
  }

  /// `image` laid over the backdrop colour `[red, green, blue]`, opaque where it was clear.
  pub fn flatten_onto(image: &mut RgbaImage, [red, green, blue]: [u8; 3]) {
    for Rgba([r, g, b, a]) in image.pixels_mut() {
      let alpha: u16 = u16::from(*a);
      let over = |channel: u8, under: u8| -> u8 {
        ((u16::from(channel) * alpha + u16::from(under) * (255 - alpha) + 127) / 255) as u8
      };

      *r = over(*r, red);
      *g = over(*g, green);
      *b = over(*b, blue);
      *a = 255;
    }
  }

  /// A capture's PNG of the screenshots folder, however large: a map 16384 pixels a side takes a gigabyte decoded,
  /// twice the decoder's own limit.
  ///
  /// # Errors
  ///
  /// Returns an error when it cannot be read or decoded.
  pub fn read_capture(source: &Path) -> TauriResult<RgbaImage> {
    let bytes: Vec<u8> = fs::read(source).map_err(|error| format!("Failed to read '{}': {error}", source.display()))?;
    let mut reader: ImageReader<Cursor<Vec<u8>>> = ImageReader::with_format(Cursor::new(bytes), ImageFormat::Png);

    reader.no_limits();

    Ok(
      reader
        .decode()
        .map_err(|error| format!("Failed to decode '{}': {error}", source.display()))?
        .to_rgba8(),
    )
  }

  /// `image` as a PNG, to be written to `target`.
  ///
  /// # Errors
  ///
  /// Returns an error when it cannot be encoded.
  pub fn encode_png(image: &RgbaImage, target: &Path) -> TauriResult<Vec<u8>> {
    let mut png: Vec<u8> = Vec::new();

    image
      .write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
      .map_err(|error| format!("Failed to encode '{}': {error}", target.display()))?;

    Ok(png)
  }

  /// `image` as an uncompressed 32 bit TGA, as `CDemoRecord` writes its own, to be written to `target`.
  ///
  /// # Errors
  ///
  /// Returns an error when it cannot be encoded.
  pub fn encode_tga(image: &RgbaImage, target: &Path) -> TauriResult<Vec<u8>> {
    let mut tga: Vec<u8> = Vec::new();

    TgaEncoder::new(&mut tga)
      .disable_rle()
      .write_image(image.as_raw(), image.width(), image.height(), ExtendedColorType::Rgba8)
      .map_err(|error| format!("Failed to encode '{}': {error}", target.display()))?;

    Ok(tga)
  }
}
