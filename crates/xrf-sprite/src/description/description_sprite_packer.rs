use std::path::PathBuf;
use std::time::Instant;

use image::{GenericImage, ImageBuffer, Rgba, RgbaImage};
use xrf_dds::DdsFile;
use xrf_error::{MapXrfErr, XrfError, XrfResult};
use xrf_extension::XrayExtension;
use xrf_utils::{HostDestination, HostRelativePath, assert_equal, format_path};

use crate::description::{
  DescriptionSpriteOptions, DescriptionSpriteResult, TextureFileDescriptor, XmlDescriptionCollection,
};
use crate::report::SpriteNotice;
use xrf_texture::ImageFileWriter;

pub struct DescriptionSpritePacker {}

impl DescriptionSpritePacker {
  /// Packs the sheets the description declares, every one or those `options` names, and answers what was written,
  /// handing whatever it passed over or noticed to `on_notice` as it is met.
  pub fn pack_xml_descriptions(
    options: &DescriptionSpriteOptions,
    mut on_notice: impl FnMut(SpriteNotice),
  ) -> XrfResult<DescriptionSpriteResult> {
    let started_at: Instant = Instant::now();
    let description: XmlDescriptionCollection = XmlDescriptionCollection::get_descriptions(options, &mut on_notice)?;
    let mut result: DescriptionSpriteResult = DescriptionSpriteResult::default();

    let selected: Vec<&TextureFileDescriptor> = description.select_files(options)?;

    // Every name is checked before the first sheet is written, so a refused one leaves no partial output behind.
    for file in &selected {
      file.to_sprite_paths()?;
    }

    let packing: xrf_job::JobScope = options.job.enter(
      crate::job_phases::SPRITE_PHASE_PACK_DESCRIPTIONS,
      Some(selected.len() as u64),
    );

    for file in selected {
      packing.advance();
      options.job.set_detail(Some(file.name.clone()));

      result = result.merged(Self::pack_xml_description(options, file, &mut on_notice)?);
    }

    options.job.set_detail(None);

    Ok(DescriptionSpriteResult {
      duration: started_at.elapsed(),
      ..result
    })
  }

  /// Packs one described sheet, answering how many of its sprites were placed and how many could not be read.
  pub fn pack_xml_description(
    options: &DescriptionSpriteOptions,
    file: &TextureFileDescriptor,
    on_notice: &mut impl FnMut(SpriteNotice),
  ) -> XrfResult<DescriptionSpriteResult> {
    let relative_path: HostRelativePath = file.to_host_relative_path()?;
    let sprite_paths: Vec<HostRelativePath> = file.to_sprite_paths()?;
    let full_name: PathBuf = options
      .base
      .join(relative_path.with_extension(XrayExtension::Dds.as_str()));

    let (width, height) = file.get_dimension_boundaries();
    let mut result: ImageBuffer<Rgba<u8>, Vec<u8>> = RgbaImage::new(width, height);
    let mut placed: u32 = 0;
    let mut skipped: u32 = 0;

    for (texture, texture_relative_path) in file.sprites.iter().zip(&sprite_paths) {
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
            .map_texture_processing_err()?;

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

          on_notice(SpriteNotice::SpriteUnreadable {
            id: texture.id.clone(),
            file: file.name.clone(),
            sheet: full_name.clone(),
            reason: error.to_string(),
          });

          skipped += 1;
        }
      }
    }

    let sheets: HostDestination = HostDestination::new(&options.output_path, options.link_policy);

    sheets.create_root()?;

    let destination: PathBuf = sheets.prepare_file(&relative_path.with_extension(XrayExtension::Dds.as_str()))?;

    if let Some(replaced) =
      ImageFileWriter::compare_replaced_dds(&destination, width, height, ImageFileWriter::UI_MIPMAP_LEVELS)
    {
      on_notice(SpriteNotice::SheetReshaped {
        path: destination.clone(),
        width,
        height,
        mipmap_levels: ImageFileWriter::UI_MIPMAP_LEVELS,
        replaced,
      });
    }

    ImageFileWriter::write_dds(
      &destination,
      &result,
      options.dds_compression_format,
      ImageFileWriter::UI_MIPMAPS,
    )?;

    Ok(DescriptionSpriteResult::written(placed, skipped))
  }
}
