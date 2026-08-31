use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use image::{DynamicImage, GenericImage, ImageBuffer, ImageReader, Rgba};
use xrf_dds::DdsFile;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;
use xrf_vfs::XrayLogicalPath;

use crate::constants::{
  DDS_EXTENSION, EXTENSIONS_DIRECTORY, LTX_PATH_EXTENSION_MARKER, LTX_PATH_EXTENSION_MARKER_PREFIX,
  LTX_PATH_GAMEDATA_MARKER, LTX_PATH_GAMEDATA_MARKER_PREFIX, PNG_EXTENSION, RESOURCES_DIRECTORY, TEXTURES_DIRECTORY,
  UI_MIPMAP_LEVELS, UI_MIPMAPS,
};
use crate::data::InventorySpriteDescriptor;
use crate::utils::{fit_image_into_bounds, warn_on_reshaped_ui_dds};
use crate::{PackEquipmentOptions, PackEquipmentResult, save_image_as_ui_dds};

pub struct PackEquipmentProcessor {}

impl PackEquipmentProcessor {
  pub fn pack_sprites(options: PackEquipmentOptions) -> XrfResult<PackEquipmentResult> {
    let started_at: Instant = Instant::now();

    let mut count: u32 = 0;
    let mut outcome: xrf_job::JobOutcome = xrf_job::JobOutcome::Completed;
    let mut skipped_sections: Vec<&str> = Vec::new();
    let mut image: ImageBuffer<Rgba<u8>, Vec<u8>> =
      InventorySpriteDescriptor::create_equipment_sprite_base_for_ltx(&options.ltx)?;

    // Variants such as `_nimble`, `_snag` and the `pri_a15_` quest copies inherit their base weapon's
    // grid position, so several sections legitimately share one slot with inheritance.
    let mut occupied_slots: HashMap<(u32, u32), (String, Vec<u8>)> = HashMap::new();

    let packing: xrf_job::JobScope = options.job.enter(
      crate::job_phases::TEXTURE_PHASE_PACK_SPRITES,
      Some(options.ltx.sections.len() as u64),
    );

    for (section_name, section) in &options.ltx.sections {
      // Between sections: the sheet is one image written once at the end, so stopping here leaves nothing on disk
      // rather than a half-drawn sprite atlas.
      if options.job.is_cancelled() {
        outcome = xrf_job::JobOutcome::Cancelled;

        break;
      }

      packing.advance();

      let Some(sprite_descriptor) = InventorySpriteDescriptor::new_optional_from_section(section_name, section) else {
        continue;
      };

      let Some((sprite_path, sprite)) = Self::read_sprite(&options, &sprite_descriptor)? else {
        skipped_sections.push(section_name);
        continue;
      };

      let (x, y, w, h) = sprite_descriptor.get_boundaries();

      xrf_output::verbose!(
        options.output,
        "Packing icon: '{}':({}:{};{}x{}) as ({}:{};{}x{}), src: {}x{}, {}",
        sprite_descriptor.section,
        sprite_descriptor.x,
        sprite_descriptor.y,
        sprite_descriptor.w,
        sprite_descriptor.h,
        x,
        y,
        w,
        h,
        sprite.width(),
        sprite.height(),
        format_path(&sprite_path),
      );

      Self::warn_on_conflicting_slot(
        &options,
        &mut occupied_slots,
        (x, y),
        &sprite_descriptor.section,
        &sprite,
      );

      image.copy_from(&sprite, x, y)?;
      count += 1;
    }

    Self::assert_every_section_has_an_icon(&options, &skipped_sections)?;

    warn_on_reshaped_ui_dds(
      &options.output,
      &options.output_path,
      image.width(),
      image.height(),
      UI_MIPMAP_LEVELS,
    );

    save_image_as_ui_dds(&options.output_path, &image, options.dds_compression_format, UI_MIPMAPS)?;

    xrf_output::info!(
      options.output,
      "Packed {} icons in {} format",
      count,
      options.dds_compression_format
    );

    Ok(PackEquipmentResult {
      outcome,
      duration: started_at.elapsed(),
      saved_at: options.output_path.clone(),
      saved_width: image.width(),
      saved_height: image.height(),
      packed_count: count,
      skipped_count: skipped_sections.len() as u32,
    })
  }

  /// Warn when a slot already holds different art, which means one section's icon is being discarded.
  ///
  /// Sharing a slot is normal and harmless while the art matches, so only a genuine difference is
  /// reported. Without this, updating a weapon's icon and forgetting its variants looks like it worked:
  /// the pack succeeds, the count is unchanged, and the variant packed later quietly wins.
  fn warn_on_conflicting_slot(
    options: &PackEquipmentOptions,
    occupied_slots: &mut HashMap<(u32, u32), (String, Vec<u8>)>,
    slot: (u32, u32),
    section: &str,
    sprite: &DynamicImage,
  ) {
    let bytes: Vec<u8> = sprite.to_rgba8().into_raw();

    match occupied_slots.get(&slot) {
      Some((previous_section, previous_bytes)) if previous_bytes != &bytes => {
        xrf_output::warning!(
          options.output,
          "Slot {}:{} is claimed by '{}' and '{}' with different icons, only the last one packed survives",
          slot.0,
          slot.1,
          previous_section,
          section
        );
      }
      _ => {}
    }

    occupied_slots.insert(slot, (String::from(section), bytes));
  }

  /// Fail once with every section that declares inventory grid coordinates but has no icon to pack.
  fn assert_every_section_has_an_icon(options: &PackEquipmentOptions, skipped_sections: &[&str]) -> XrfResult {
    if !options.is_strict || skipped_sections.is_empty() {
      return Ok(());
    }

    Err(XrfError::new_texture_processing_error(format!(
      "Expected an icon to exist for each of the {} sections declaring inv_grid_* fields, found none for: {}",
      skipped_sections.len(),
      skipped_sections.join(", ")
    )))
  }

  pub fn read_sprite(
    options: &PackEquipmentOptions,
    sprite: &InventorySpriteDescriptor,
  ) -> XrfResult<Option<(PathBuf, DynamicImage)>> {
    let (_, _, w, h) = sprite.get_boundaries();
    let sprite_path: PathBuf = Self::read_sprite_path(options, sprite)?;

    match Self::read_sprite_from_path(&sprite_path, w, h) {
      Ok(icon) => Ok(Some((sprite_path, icon))),
      Err(error) => {
        xrf_output::warning!(
          options.output,
          "Skip icon {} / '{}', reason: {}",
          format_path(&sprite_path),
          sprite.section,
          error
        );

        Ok(None)
      }
    }
  }

  /// Read rescaled png or dds icon to inject into one large equipment file.
  pub fn read_sprite_from_path(path: &Path, width: u32, height: u32) -> XrfResult<DynamicImage> {
    let image: DynamicImage = if path.extension().is_some_and(|extension| extension.eq(PNG_EXTENSION)) {
      ImageReader::open(path)?.decode()?
    } else {
      DdsFile::read_from_path(path)?.decode_rgba(0)?.into()
    };

    fit_image_into_bounds(image, width, height, path)
  }

  /// Read equipment icon from custom path defined in ltx config directory.
  pub fn read_sprite_path(
    options: &PackEquipmentOptions,
    descriptor: &InventorySpriteDescriptor,
  ) -> XrfResult<PathBuf> {
    match descriptor.custom_icon.as_deref() {
      None => {
        let png_path: PathBuf = options.source.join(format!("{}.{}", descriptor.section, PNG_EXTENSION));

        if png_path.exists() {
          Ok(png_path)
        } else {
          Ok(options.source.join(format!("{}.{}", descriptor.section, DDS_EXTENSION)))
        }
      }
      Some(custom_path) => {
        // Handle custom gamedata source.
        if let Some(gamedata) = &options.gamedata {
          if custom_path.starts_with(LTX_PATH_GAMEDATA_MARKER) {
            Self::resolve_logical_path(
              gamedata,
              custom_path.strip_prefix(LTX_PATH_GAMEDATA_MARKER_PREFIX).unwrap(),
            )
          } else {
            Self::resolve_logical_path(&gamedata.join(TEXTURES_DIRECTORY), custom_path)
          }
          // Handle ~ path for xrf / system.ltx
        } else if custom_path.starts_with(LTX_PATH_GAMEDATA_MARKER) {
          Self::resolve_logical_path(
            &options
              .ltx
              .directory
              .as_ref()
              .unwrap()
              .join("..")
              .join("..")
              .join(RESOURCES_DIRECTORY),
            custom_path.strip_prefix(LTX_PATH_GAMEDATA_MARKER_PREFIX).unwrap(),
          )
          // Handle relative path for xrf / system.ltx extensions
        } else if custom_path.starts_with(LTX_PATH_EXTENSION_MARKER) {
          Self::resolve_logical_path(
            &options
              .ltx
              .directory
              .as_ref()
              .unwrap()
              .join("..")
              .join(EXTENSIONS_DIRECTORY),
            custom_path.strip_prefix(LTX_PATH_EXTENSION_MARKER_PREFIX).unwrap(),
          )
          // Handle relative path for xrf / system.ltx
        } else {
          Self::resolve_logical_path(
            &options
              .ltx
              .directory
              .as_ref()
              .unwrap()
              .join("..")
              .join("..")
              .join(RESOURCES_DIRECTORY)
              .join(TEXTURES_DIRECTORY),
            custom_path,
          )
        }
      }
    }
  }

  /// Resolves an X-Ray path below a trusted host root without letting engine separators leak into host I/O.
  fn resolve_logical_path(root: &Path, logical_path: &str) -> XrfResult<PathBuf> {
    Ok(root.join(XrayLogicalPath::new(logical_path)?.to_host_relative_path()))
  }
}
