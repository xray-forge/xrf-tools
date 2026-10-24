use std::path::PathBuf;
use std::time::Instant;

use image::{GenericImage, ImageBuffer, Rgba, RgbaImage};
use xrf_dds::DdsFile;
use xrf_error::{XrfError, XrfResult};
use xrf_extension::XrayExtension;
use xrf_utils::{HostDestination, HostRelativePath, assert_equal, format_path};

use crate::description::{
  SpriteDescriptionOptions, SpriteDescriptionResult, TextureFileDescriptor, XmlDescriptionCollection,
};
use crate::image_file::{UI_MIPMAP_LEVELS, UI_MIPMAPS, save_image_as_ui_dds, warn_on_reshaped_ui_dds};

pub struct PackDescriptionProcessor {}

impl PackDescriptionProcessor {
  /// Packs the sheets the description declares, every one or those `options` names, and answers what was written.
  pub fn pack_xml_descriptions(options: &SpriteDescriptionOptions) -> XrfResult<SpriteDescriptionResult> {
    let started_at: Instant = Instant::now();
    let description: XmlDescriptionCollection = XmlDescriptionCollection::get_descriptions(options)?;
    let mut result: SpriteDescriptionResult = SpriteDescriptionResult::default();

    let selected: Vec<&TextureFileDescriptor> = description.select_files(options)?;

    // Every name is checked before the first sheet is written, so a refused one leaves no partial output behind.
    for file in &selected {
      file.to_sprite_paths()?;
    }

    xrf_output::info!(options.output, "Packing for {} files", selected.len());

    let packing: xrf_job::JobScope = options.job.enter(
      crate::job_phases::TEXTURE_PHASE_PACK_DESCRIPTIONS,
      Some(selected.len() as u64),
    );

    for file in selected {
      packing.advance();

      result = result.merged(Self::pack_xml_description(options, file)?);
    }

    xrf_output::info!(options.output, "Packed {} files", result.files);

    Ok(SpriteDescriptionResult {
      duration: started_at.elapsed(),
      ..result
    })
  }

  /// Packs one described sheet, answering how many of its sprites were placed and how many could not be read.
  pub fn pack_xml_description(
    options: &SpriteDescriptionOptions,
    file: &TextureFileDescriptor,
  ) -> XrfResult<SpriteDescriptionResult> {
    let relative_path: HostRelativePath = file.to_host_relative_path()?;
    let sprite_paths: Vec<HostRelativePath> = file.to_sprite_paths()?;
    let full_name: PathBuf = options
      .base
      .join(relative_path.with_extension(XrayExtension::Dds.as_str()));

    let (width, height) = file.get_dimension_boundaries();
    let mut result: ImageBuffer<Rgba<u8>, Vec<u8>> = RgbaImage::new(width, height);
    let mut placed: u32 = 0;
    let mut skipped: u32 = 0;

    xrf_output::verbose!(
      options.output,
      "Packing file {} ({width}x{height})",
      format_path(&full_name)
    );

    for (texture, texture_relative_path) in file.sprites.iter().zip(&sprite_paths) {
      xrf_output::verbose!(
        options.output,
        "Packing texture {} -> {} at [x:{}, y:{}, w:{}, h:{}]",
        format_path(&full_name),
        texture.id,
        texture.x,
        texture.y,
        texture.w,
        texture.h
      );

      let texture_path: PathBuf = options.base.join(texture_relative_path);

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

          placed += 1;
        }
        Err(error) => {
          if options.is_strict {
            return Err(XrfError::new_texture_processing_error(format!(
              "Failed to read texture dds {} for {} ({}): {}",
              texture.id,
              file.name,
              format_path(&full_name),
              error
            )));
          }

          xrf_output::warning!(
            options.output,
            "Failed to read texture dds {} for {} ({}): {}",
            texture.id,
            file.name,
            format_path(&full_name),
            error
          );

          skipped += 1;
        }
      }
    }

    let sheets: HostDestination = HostDestination::new(&options.output_path, options.link_policy);

    sheets.create_root()?;

    let destination: PathBuf = sheets.prepare_file(&relative_path.with_extension(XrayExtension::Dds.as_str()))?;

    xrf_output::verbose!(options.output, "Saving file: {}", format_path(&destination));

    warn_on_reshaped_ui_dds(&options.output, &destination, width, height, UI_MIPMAP_LEVELS);

    save_image_as_ui_dds(&destination, &result, options.dds_compression_format, UI_MIPMAPS)?;

    Ok(SpriteDescriptionResult::written(placed, skipped))
  }
}
