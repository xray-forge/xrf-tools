use std::path::PathBuf;
use std::time::Instant;

use image::{GenericImageView, RgbaImage};
use rayon::prelude::*;
use xrf_dds::{DdsFile, DdsMipmaps};
use xrf_error::{XrfError, XrfResult};
use xrf_extension::XrayExtension;
use xrf_output::{OutputOptions, OutputSequence, OutputSlot};
use xrf_utils::{HostDestination, HostRelativePath, format_path};

use crate::description::{
  DescriptionSpriteOptions, DescriptionSpriteResult, TextureFileDescriptor, XmlDescriptionCollection,
};
use crate::image_file::save_image_as_ui_dds;

pub struct DescriptionSpriteUnpacker {}

impl DescriptionSpriteUnpacker {
  /// Cuts the sprites out of the sheets the description declares, every one or those `options` names, and answers
  /// what was written.
  pub fn unpack_xml_descriptions(options: &DescriptionSpriteOptions) -> XrfResult<DescriptionSpriteResult> {
    let started_at: Instant = Instant::now();
    let description: XmlDescriptionCollection = XmlDescriptionCollection::get_descriptions(options)?;
    let selected: Vec<&TextureFileDescriptor> = description.select_files(options)?;

    // Every name is checked before the first sprite is written, so a refused one leaves no partial output behind.
    for file in &selected {
      file.to_sprite_paths()?;
    }

    xrf_output::info!(options.output, "Unpacking for {} files", selected.len());

    let unpacking: xrf_job::JobScope = options.job.enter(
      crate::job_phases::TEXTURE_PHASE_UNPACK_DESCRIPTIONS,
      Some(selected.len() as u64),
    );

    // Each file logs into its listed position and the sequence releases them in selection order rather than in the
    // order the workers finished. A file that fails still takes its turn, because the slot commits as the error
    // unwinds past it — which is what keeps output identical however wide the run is.
    let sequence: OutputSequence = OutputSequence::new(&options.output, selected.len());

    let result: DescriptionSpriteResult = selected
      .par_iter()
      .enumerate()
      .map(|(index, file)| {
        let slot: OutputSlot = sequence.new_slot(index);
        let unpacked: DescriptionSpriteResult = Self::unpack_xml_description(options, slot.get_output(), file)?;

        unpacking.advance();

        Ok::<DescriptionSpriteResult, XrfError>(unpacked)
      })
      .try_reduce(DescriptionSpriteResult::default, |left, right| Ok(left.merged(right)))?;

    xrf_output::info!(options.output, "Unpacked {} files", result.files);

    Ok(DescriptionSpriteResult {
      duration: started_at.elapsed(),
      ..result
    })
  }

  /// Cuts the sprites out of one described sheet, answering how many were written and how many did not fit; a sheet
  /// that cannot be read is passed over whole outside strict mode.
  pub fn unpack_xml_description(
    options: &DescriptionSpriteOptions,
    output: &OutputOptions,
    file: &TextureFileDescriptor,
  ) -> XrfResult<DescriptionSpriteResult> {
    let relative_path: HostRelativePath = file.to_host_relative_path()?;
    let sprite_paths: Vec<HostRelativePath> = file.to_sprite_paths()?;
    let full_name: PathBuf = options
      .base
      .join(relative_path.with_extension(XrayExtension::Dds.as_str()));
    let destination: HostDestination = HostDestination::new(&options.output_path, options.link_policy);

    xrf_output::verbose!(output, "Unpacking {}", format_path(&full_name));

    let dds: RgbaImage = match DdsFile::read_from_path(&full_name).and_then(|dds| dds.decode_rgba(0)) {
      Ok(dds) => dds,
      Err(_) if options.is_strict => {
        return Err(XrfError::new_texture_processing_error(format!(
          "Could not find file for texture unpacking: {}",
          format_path(&full_name)
        )));
      }
      Err(error) => {
        xrf_output::warning!(
          output,
          "Skip file {}, not able to read: {}",
          format_path(&full_name),
          error
        );

        return Ok(DescriptionSpriteResult::skipped());
      }
    };

    destination.create_root()?;
    destination.create_directory(&relative_path)?;

    let mut written: u32 = 0;
    let mut skipped: u32 = 0;

    for (sprite, sprite_path) in file.sprites.iter().zip(&sprite_paths) {
      xrf_output::verbose!(output, "Unpacking {} -> {}", format_path(&full_name), sprite.id);

      let (max_x, max_y) = sprite.get_dimension_boundaries();

      if max_x > dds.width() || max_y > dds.height() {
        if options.is_strict {
          return Err(XrfError::new_texture_processing_error(format!(
            "Unexpected texture '{}' (x:{}, y:{}) boundaries are bigger than source DDS file ({}x{} - {})",
            sprite.id,
            max_x,
            max_y,
            dds.width(),
            dds.height(),
            format_path(&full_name)
          )));
        }

        xrf_output::warning!(
          output,
          "[WARN] - exceeding sprite size '{}' (x:{}, y:{}) ({}x{} - {})",
          sprite.id,
          max_x,
          max_y,
          dds.width(),
          dds.height(),
          format_path(&full_name)
        );

        skipped += 1;

        continue;
      }

      // Unpacked sprites are packing input read at their base level, so a mip chain would only
      // cost space.
      save_image_as_ui_dds(
        &destination.prepare_file(sprite_path)?,
        &dds.view(sprite.x, sprite.y, sprite.w, sprite.h).to_image(),
        options.dds_compression_format,
        DdsMipmaps::Disabled,
      )?;

      written += 1;
    }

    Ok(DescriptionSpriteResult::written(written, skipped))
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;

  use image::RgbaImage;
  use xrf_dds::{DdsMipmaps, ImageFormat};
  use xrf_error::XrfError;
  use xrf_test_utils::utils::{build_absolute_generated_test_resource_path, create_generated_test_resource_dir};

  use super::DescriptionSpriteUnpacker;
  use crate::image_file::save_image_as_ui_dds;
  use crate::{DescriptionSpriteOptions, DescriptionSpriteResult, TextureFileDescriptor};

  fn options_for_missing_source(is_strict: bool) -> DescriptionSpriteOptions {
    let root: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .parent()
      .and_then(|path| path.parent())
      .expect("xrf-texture to be inside the workspace crates directory")
      .join("target")
      .join("test-resources")
      .join(format!("xrf-texture-missing-description-{}", std::process::id()));

    DescriptionSpriteOptions {
      job: Default::default(),
      description: root.join("description.xml"),
      base: root.join("source"),
      output: Default::default(),
      output_path: root.join("output"),
      dds_compression_format: ImageFormat::BC3RgbaUnorm,
      files: Vec::new(),
      is_strict,
      link_policy: Default::default(),
    }
  }

  #[test]
  fn skips_an_unreadable_sheet_in_non_strict_mode() {
    let options: DescriptionSpriteOptions = options_for_missing_source(false);
    let file: TextureFileDescriptor = TextureFileDescriptor::new(r"ui\missing");

    assert_eq!(
      DescriptionSpriteUnpacker::unpack_xml_description(&options, &options.output, &file)
        .expect("non-strict unpacking to skip an unreadable sheet"),
      DescriptionSpriteResult::skipped()
    );
    assert!(
      !options
        .output_path
        .join(file.to_host_relative_path().expect("valid logical path"))
        .exists()
    );
  }

  #[test]
  fn rejects_an_unreadable_sheet_in_strict_mode() {
    let options: DescriptionSpriteOptions = options_for_missing_source(true);
    let file: TextureFileDescriptor = TextureFileDescriptor::new(r"ui\missing");

    assert!(DescriptionSpriteUnpacker::unpack_xml_description(&options, &options.output, &file).is_err());
    assert!(
      !options
        .output_path
        .join(file.to_host_relative_path().expect("valid logical path"))
        .exists()
    );
  }

  const SENTINEL: &[u8] = b"sentinel";

  /// A readable 8x4 sheet at `base/sheet.dds`, a description of `ids` over it, and a sentinel beside `out`.
  fn options_for_sheet(name: &str, ids: &[&str], is_strict: bool) -> DescriptionSpriteOptions {
    let root: PathBuf = create_generated_test_resource_dir(&format!("description-unpack/{name}")).expect("sheet root");

    fs::create_dir_all(root.join("base")).expect("sheet directory");
    fs::write(root.join("victim.dds"), SENTINEL).expect("sentinel beside the destination");
    save_image_as_ui_dds(
      &root.join("base").join("sheet.dds"),
      &RgbaImage::new(8, 4),
      ImageFormat::BC3RgbaUnorm,
      DdsMipmaps::Disabled,
    )
    .expect("sheet");

    let textures: String = ids
      .iter()
      .enumerate()
      .map(|(index, id)| {
        format!(
          "<texture id=\"{id}\" x=\"{}\" y=\"0\" width=\"4\" height=\"4\" />",
          (index % 2) * 4
        )
      })
      .collect();

    fs::write(
      root.join("description.xml"),
      format!("<w><file name=\"sheet\">{textures}</file></w>"),
    )
    .expect("description");

    DescriptionSpriteOptions {
      job: Default::default(),
      description: root.join("description.xml"),
      base: root.join("base"),
      output: Default::default(),
      output_path: root.join("out"),
      dds_compression_format: ImageFormat::BC3RgbaUnorm,
      files: Vec::new(),
      is_strict,
      link_policy: Default::default(),
    }
  }

  fn root_of(options: &DescriptionSpriteOptions) -> PathBuf {
    options.base.parent().expect("a test root").to_path_buf()
  }

  #[test]
  fn refuses_a_sprite_id_that_leaves_the_destination() {
    let absolute: String = build_absolute_generated_test_resource_path("description-unpack/absolute/victim")
      .display()
      .to_string();

    for (index, id) in [
      "../../victim",
      r"..\..\victim",
      r"nested\..\..\..\victim",
      "c:victim",
      absolute.as_str(),
    ]
    .into_iter()
    .enumerate()
    {
      for is_strict in [false, true] {
        // A valid sprite is listed first, so a run that wrote before it checked would leave it behind.
        let options: DescriptionSpriteOptions =
          options_for_sheet(&format!("escape-{index}-{is_strict}"), &["ui_valid", id], is_strict);
        let root: PathBuf = root_of(&options);

        assert!(
          matches!(
            DescriptionSpriteUnpacker::unpack_xml_descriptions(&options),
            Err(XrfError::Invalid { .. })
          ),
          "sprite id '{id}' must be refused by name"
        );
        assert_eq!(
          fs::read(root.join("victim.dds")).expect("sentinel"),
          SENTINEL,
          "'{id}' overwrote the sentinel"
        );
        assert!(!root.join("out").exists(), "'{id}' left earlier output behind");
      }
    }
  }

  #[test]
  fn writes_each_sprite_under_its_id_as_spelled() {
    let options: DescriptionSpriteOptions = options_for_sheet("valid", &["ui_icon_PDA", "ui icon (old)"], true);
    let root: PathBuf = root_of(&options);

    let result: DescriptionSpriteResult = DescriptionSpriteUnpacker::unpack_xml_descriptions(&options).expect("unpack");

    assert_eq!((result.files, result.sprites, result.skipped_sprites), (1, 2, 0));
    assert!(root.join("out").join("sheet").join("ui_icon_PDA.dds").is_file());
    assert!(root.join("out").join("sheet").join("ui icon (old).dds").is_file());
    assert_eq!(fs::read(root.join("victim.dds")).expect("sentinel"), SENTINEL);
  }
}
