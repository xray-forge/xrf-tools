use std::fs;
use std::path::PathBuf;

use image::{GenericImage, ImageBuffer, Rgba, RgbaImage};
use xrf_dds::DdsFile;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::assert_equal;

use crate::constants::{DDS_EXTENSION, UI_MIPMAP_LEVELS, UI_MIPMAPS};
use crate::data::TextureFileDescriptor;
use crate::description::{PackDescriptionOptions, XmlDescriptionCollection};
use crate::save_image_as_ui_dds;
use crate::utils::warn_on_reshaped_ui_dds;

pub struct PackDescriptionProcessor {}

impl PackDescriptionProcessor {
  /// Pack list of xml files by options.
  pub fn pack_xml_descriptions(options: &PackDescriptionOptions) -> XrfResult {
    let description: XmlDescriptionCollection = XmlDescriptionCollection::get_descriptions(options)?;
    let mut count: u32 = 0;

    let selected: Vec<&TextureFileDescriptor> = description.select_files(options)?;

    xrf_output::info!(options.output, "Packing for {} files", selected.len());

    for file in selected {
      if Self::pack_xml_description(options, file)? {
        count += 1;
      }
    }

    xrf_output::info!(options.output, "Packed {count} files");

    Ok(())
  }

  pub fn pack_xml_description(options: &PackDescriptionOptions, file: &TextureFileDescriptor) -> XrfResult<bool> {
    let relative_path: PathBuf = file.to_host_relative_path()?;
    let full_name: PathBuf = options.base.join(relative_path.with_extension(DDS_EXTENSION));

    let (width, height) = file.get_dimension_boundaries();
    let mut result: ImageBuffer<Rgba<u8>, Vec<u8>> = RgbaImage::new(width, height);

    xrf_output::verbose!(
      options.output,
      "Packing file {} ({width}x{height})",
      full_name.display()
    );

    for texture in &file.sprites {
      xrf_output::verbose!(
        options.output,
        "Packing texture {} -> {} at [x:{}, y:{}, w:{}, h:{}]",
        full_name.display(),
        texture.id,
        texture.x,
        texture.y,
        texture.w,
        texture.h
      );

      let texture_path: PathBuf = options
        .base
        .join(&relative_path)
        .join(format!("{}.{}", texture.id, DDS_EXTENSION));

      match DdsFile::read_from_path(&texture_path).and_then(|dds| dds.decode_rgba(0)) {
        Ok(texture_dds) => {
          assert_equal(
            texture_dds.width(),
            texture.w,
            "XML file texture width and actual DDS size should match",
          )?;
          assert_equal(
            texture_dds.height(),
            texture.h,
            "XML file texture height and actual DDS size should match",
          )?;

          result
            .copy_from(&texture_dds, texture.x, texture.y)
            .map_err(|error| XrfError::new_texture_processing_error(error.to_string()))?;
        }
        Err(error) => {
          if options.is_strict {
            return Err(XrfError::new_texture_processing_error(format!(
              "Failed to read texture dds {} for {} ({}): {}",
              texture.id,
              file.name,
              full_name.display(),
              error
            )));
          } else {
            xrf_output::warning!(
              options.output,
              "Failed to read texture dds {} for {} ({}): {}",
              texture.id,
              file.name,
              full_name.display(),
              error
            )
          }
        }
      }
    }

    let destination: PathBuf = options.output_path.join(relative_path.with_extension(DDS_EXTENSION));

    xrf_output::verbose!(options.output, "Saving file: {}", destination.display());

    warn_on_reshaped_ui_dds(&options.output, &destination, width, height, UI_MIPMAP_LEVELS);

    if let Some(parent) = destination.parent().filter(|parent| !parent.as_os_str().is_empty()) {
      fs::create_dir_all(parent)?;
    }

    save_image_as_ui_dds(&destination, &result, options.dds_compression_format, UI_MIPMAPS)?;

    Ok(true)
  }
}
