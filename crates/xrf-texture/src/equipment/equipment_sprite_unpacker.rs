use image::GenericImageView;
use xrf_dds::DdsMipmaps;
use xrf_error::XrfResult;
use xrf_extension::XrayExtension;
use xrf_utils::{HostDestination, HostRelativePath};

use crate::equipment::{EquipmentSpriteUnpackOptions, InventorySpriteDescriptor};
use crate::image_file::save_image_as_ui_dds;

pub struct EquipmentSpriteUnpacker {}

impl EquipmentSpriteUnpacker {
  pub fn unpack_sprites(options: EquipmentSpriteUnpackOptions) -> XrfResult {
    let mut count: u32 = 0;
    let sprites: Vec<InventorySpriteDescriptor> = InventorySpriteDescriptor::new_list_from_ltx(&options.ltx);

    // Every name is checked before the first icon is written, so a refused one leaves no partial output behind.
    for sprite in &sprites {
      sprite.to_host_file_name(XrayExtension::Dds)?;
    }

    let destination: HostDestination = HostDestination::new(&options.output_path, options.link_policy);

    destination.create_root()?;

    let unpacking: xrf_job::JobScope = options.job.enter(
      crate::job_phases::TEXTURE_PHASE_UNPACK_SPRITES,
      Some(sprites.len() as u64),
    );

    for sprite in &sprites {
      unpacking.advance();

      if Self::unpack_sprite(&options, &destination, sprite)? {
        count += 1;
      }
    }

    xrf_output::info!(options.output, "Unpacked {count} icons");

    Ok(())
  }

  pub fn unpack_sprite(
    options: &EquipmentSpriteUnpackOptions,
    destination: &HostDestination,
    sprite: &InventorySpriteDescriptor,
  ) -> XrfResult<bool> {
    let (x, y, w, h) = sprite.get_boundaries();
    let relative_path: HostRelativePath = sprite.to_host_file_name(XrayExtension::Dds)?.into();

    xrf_output::verbose!(
      options.output,
      "Unpacking icon: '{}' x:{}({x}), y:{}({y}), w:{}({w}), h:{}({h})",
      sprite.section,
      sprite.x,
      sprite.y,
      sprite.w,
      sprite.h,
    );

    // todo: Respect custom icon path from LTX file here (sprite.custom_icon).

    if x.saturating_add(w) > options.source.width() || y.saturating_add(h) > options.source.height() {
      xrf_output::warning!(
        options.output,
        "Skip for possible section: '{}' - icon is out of source file bonds",
        sprite.section
      );

      Ok(false)
    } else {
      // Unpacked icons are packing input read at their base level, so a mip chain would only cost
      // space.
      save_image_as_ui_dds(
        &destination.prepare_file(&relative_path)?,
        &options.source.view(x, y, w, h).to_image(),
        options.dds_compression_format,
        DdsMipmaps::Disabled,
      )?;

      Ok(true)
    }
  }
}
