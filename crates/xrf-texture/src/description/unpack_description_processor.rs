use std::fs::create_dir_all;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use image::{GenericImageView, RgbaImage};
use rayon::prelude::*;
use xrf_dds::{DdsFile, Mipmaps};
use xrf_error::{XrfError, XrfResult};
use xrf_output::{OutputOptions, OutputSequence, OutputSlot};

use crate::constants::DDS_EXTENSION;
use crate::data::TextureFileDescriptor;
use crate::description::XmlDescriptionCollection;
use crate::{PackDescriptionOptions, save_image_as_ui_dds};

pub struct UnpackDescriptionProcessor {}

impl UnpackDescriptionProcessor {
  pub fn unpack_xml_descriptions(options: PackDescriptionOptions) -> XrfResult<()> {
    let description: XmlDescriptionCollection = XmlDescriptionCollection::get_descriptions(&options)?;
    let count: AtomicU32 = AtomicU32::new(0);
    let selected: Vec<&TextureFileDescriptor> = description.select_files(&options)?;

    xrf_output::info!(options.output, "Unpacking for {} files", selected.len());

    if options.is_parallel {
      // Files are unpacked in parallel, so each one logs into its listed position and the sequence
      // releases them in selection order rather than in the order the workers finished. A file that
      // fails still takes its turn, because the slot commits as the error unwinds past it.
      let sequence: OutputSequence = OutputSequence::new(&options.output, selected.len());

      selected.par_iter().enumerate().try_for_each(|(index, file)| {
        let slot: OutputSlot = sequence.new_slot(index);

        if Self::unpack_xml_description(&options, slot.get_output(), file)? {
          count.fetch_add(1, Ordering::Relaxed);
        }

        Ok::<(), XrfError>(())
      })?;
    } else {
      for file in selected {
        if Self::unpack_xml_description(&options, &options.output, file)? {
          count.fetch_add(1, Ordering::Relaxed);
        }
      }
    }

    xrf_output::info!(options.output, "Unpacked {} files", count.load(Ordering::Relaxed));

    Ok(())
  }

  pub fn unpack_xml_description(
    options: &PackDescriptionOptions,
    output: &OutputOptions,
    file: &TextureFileDescriptor,
  ) -> XrfResult<bool> {
    let relative_path: PathBuf = file.to_host_relative_path()?;
    let full_name: PathBuf = options.base.join(relative_path.with_extension(DDS_EXTENSION));
    let destination: PathBuf = options.output_path.join(relative_path);

    xrf_output::verbose!(output, "Unpacking {}", full_name.display());

    let dds: RgbaImage = match DdsFile::read_from_path(&full_name).and_then(|dds| dds.decode_rgba(0)) {
      Ok(dds) => dds,
      Err(_) if options.is_strict => {
        return Err(XrfError::new_texture_processing_error(format!(
          "Could not find file for texture unpacking: {}",
          full_name.display()
        )));
      }
      Err(error) => {
        xrf_output::warning!(output, "Skip file {}, not able to read: {}", full_name.display(), error);

        return Ok(false);
      }
    };

    if !destination.exists() {
      create_dir_all(&destination)?;
    }

    for sprite in &file.sprites {
      xrf_output::verbose!(output, "Unpacking {} -> {}", full_name.display(), sprite.id);

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
            full_name.display()
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
          full_name.display()
        );

        continue;
      }

      // Unpacked sprites are packing input read at their base level, so a mip chain would only
      // cost space.
      save_image_as_ui_dds(
        &destination.join(format!("{}.{}", sprite.id, DDS_EXTENSION)),
        &dds.view(sprite.x, sprite.y, sprite.w, sprite.h).to_image(),
        options.dds_compression_format,
        Mipmaps::Disabled,
      )?;
    }

    Ok(true)
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use xrf_dds::ImageFormat;

  use super::UnpackDescriptionProcessor;
  use crate::{PackDescriptionOptions, TextureFileDescriptor};

  fn options_for_missing_source(is_strict: bool) -> PackDescriptionOptions {
    let root: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .parent()
      .and_then(|path| path.parent())
      .expect("xrf-texture to be inside the workspace crates directory")
      .join("target")
      .join("test-resources")
      .join(format!("xrf-texture-missing-description-{}", std::process::id()));

    PackDescriptionOptions {
      description: root.join("description.xml"),
      base: root.join("source"),
      output: Default::default(),
      output_path: root.join("output"),
      dds_compression_format: ImageFormat::BC3RgbaUnorm,
      files: Vec::new(),
      is_strict,
      is_parallel: false,
    }
  }

  #[test]
  fn skips_an_unreadable_sheet_in_non_strict_mode() {
    let options: PackDescriptionOptions = options_for_missing_source(false);
    let file: TextureFileDescriptor = TextureFileDescriptor::new(r"ui\missing");

    assert!(
      !UnpackDescriptionProcessor::unpack_xml_description(&options, &options.output, &file)
        .expect("non-strict unpacking to skip an unreadable sheet")
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
    let options: PackDescriptionOptions = options_for_missing_source(true);
    let file: TextureFileDescriptor = TextureFileDescriptor::new(r"ui\missing");

    assert!(UnpackDescriptionProcessor::unpack_xml_description(&options, &options.output, &file).is_err());
    assert!(
      !options
        .output_path
        .join(file.to_host_relative_path().expect("valid logical path"))
        .exists()
    );
  }
}
