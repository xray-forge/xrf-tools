use image::{DynamicImage, GenericImageView, RgbaImage};
use xrf_dds::DdsFile;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::constants::PNG_EXTENSION;
use crate::crop::{CropTextureOptions, CropTextureResult};
use crate::utils::{fit_image_into_bounds, save_image_as_ui_dds, save_image_as_ui_png};

pub struct CropTextureProcessor {}

impl CropTextureProcessor {
  pub fn crop(options: &CropTextureOptions) -> XrfResult<CropTextureResult> {
    let image: RgbaImage = DdsFile::read_from_path(&options.source)?.decode_rgba(0)?;

    if options.x + options.width > image.width() || options.y + options.height > image.height() {
      return Err(XrfError::new_texture_processing_error(format!(
        "Region {}:{} {}x{} does not fit in source {} which is {}x{}",
        options.x,
        options.y,
        options.width,
        options.height,
        format_path(&options.source),
        image.width(),
        image.height()
      )));
    }

    let cropped: RgbaImage = image
      .view(options.x, options.y, options.width, options.height)
      .to_image();
    let result: RgbaImage = match (options.fit_width, options.fit_height) {
      (Some(fit_width), Some(fit_height)) => {
        xrf_output::info!(
          options.output,
          "Fitting cropped {}x{} region into {}x{}",
          options.width,
          options.height,
          fit_width,
          fit_height
        );

        fit_image_into_bounds(DynamicImage::from(cropped), fit_width, fit_height, &options.source)?.into()
      }
      _ => cropped,
    };

    if options
      .output_path
      .extension()
      .is_some_and(|extension| extension.eq(PNG_EXTENSION))
    {
      save_image_as_ui_png(&options.output_path, &result)?;
    } else {
      save_image_as_ui_dds(
        &options.output_path,
        &result,
        options.dds_compression_format,
        options.dds_mipmaps,
      )?;
    }

    Ok(CropTextureResult {
      width: result.width(),
      height: result.height(),
    })
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use image::RgbaImage;
  use xrf_dds::{DdsEncodeOptions, DdsFile, ImageFormat, Mipmaps, Quality};
  use xrf_test_utils::utils::write_generated_test_resource;

  use super::CropTextureProcessor;
  use crate::CropTextureOptions;

  fn options_for(name: &str) -> CropTextureOptions {
    let resource: String = format!("xrf-texture/crop/{name}-source.dds");
    let source: PathBuf = write_generated_test_resource(&resource, []).expect("expect scratch source");

    DdsFile::encode_rgba(
      &RgbaImage::new(8, 8),
      DdsEncodeOptions::new(ImageFormat::BC3RgbaUnorm, Quality::Slow, Mipmaps::Disabled),
    )
    .expect("expect source DDS to encode")
    .write_to_path(&source)
    .expect("expect source DDS to be written");

    CropTextureOptions {
      output_path: source.with_file_name(format!("{name}-output.dds")),
      source,
      output: Default::default(),
      x: 2,
      y: 1,
      width: 4,
      height: 3,
      fit_width: None,
      fit_height: None,
      dds_compression_format: ImageFormat::BC3RgbaUnorm,
      dds_mipmaps: Mipmaps::Disabled,
    }
  }

  #[test]
  fn crops_a_dds_through_the_texture_interface() {
    let options: CropTextureOptions = options_for("valid");
    let result = CropTextureProcessor::crop(&options).expect("expect crop to succeed");
    let output = DdsFile::read_from_path(&options.output_path).expect("expect cropped DDS to parse");

    assert_eq!((result.width, result.height), (4, 3));
    assert_eq!((output.metadata().width, output.metadata().height), (4, 3));
  }

  #[test]
  fn rejects_a_region_outside_the_source() {
    let mut options: CropTextureOptions = options_for("outside");

    options.x = 7;

    assert!(CropTextureProcessor::crop(&options).is_err());
    assert!(!options.output_path.exists());
  }
}
