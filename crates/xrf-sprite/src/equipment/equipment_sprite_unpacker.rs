use image::GenericImageView;
use xrf_dds::DdsMipmaps;
use xrf_error::XrfResult;
use xrf_extension::XrayExtension;
use xrf_texture::ImageFileWriter;
use xrf_utils::{HostDestination, HostRelativePath};

use crate::equipment::{EquipmentSpriteUnpackOptions, InventorySpriteDescriptor};
use crate::report::{SpriteListener, SpriteNotice};

pub struct EquipmentSpriteUnpacker {}

impl EquipmentSpriteUnpacker {
  /// Cuts every declared icon out of the sheet into a file named by its section, answering how many it wrote and
  /// handing each one it passed over to the options' listener.
  pub fn unpack_sprites(options: EquipmentSpriteUnpackOptions) -> XrfResult<u32> {
    let listener: Option<SpriteListener> = options.listener.clone();
    let mut on_notice = |notice: SpriteNotice| {
      if let Some(listener) = &listener {
        listener(&notice);
      }
    };
    let mut count: u32 = 0;
    let sprites: Vec<InventorySpriteDescriptor> = InventorySpriteDescriptor::new_list_from_ltx(&options.ltx);

    // Every name is checked before the first icon is written, so a refused one leaves no partial output behind.
    for sprite in &sprites {
      sprite.to_host_file_name(XrayExtension::Dds)?;
    }

    let destination: HostDestination = HostDestination::new(&options.output_path, options.link_policy);

    destination.create_root()?;

    let unpacking: xrf_job::JobScope = options.job.enter(
      crate::job_phases::SPRITE_PHASE_UNPACK_EQUIPMENT,
      Some(sprites.len() as u64),
    );

    for sprite in &sprites {
      unpacking.advance();
      options.job.set_detail(Some(sprite.section.clone()));

      if Self::unpack_sprite(&options, &destination, sprite, &mut on_notice)? {
        count += 1;
      }
    }

    options.job.set_detail(None);

    Ok(count)
  }

  /// Cuts one icon out of the sheet, answering whether it was written; one outside the sheet is noticed and skipped.
  fn unpack_sprite(
    options: &EquipmentSpriteUnpackOptions,
    destination: &HostDestination,
    sprite: &InventorySpriteDescriptor,
    on_notice: &mut impl FnMut(SpriteNotice),
  ) -> XrfResult<bool> {
    let (x, y, w, h) = sprite.get_boundaries();
    let relative_path: HostRelativePath = sprite.to_host_file_name(XrayExtension::Dds)?.into();

    // todo: Respect custom icon path from LTX file here (sprite.custom_icon).

    if x.saturating_add(w) > options.source.width() || y.saturating_add(h) > options.source.height() {
      on_notice(SpriteNotice::IconOutsideSheet {
        section: sprite.section.clone(),
      });

      Ok(false)
    } else {
      // Unpacked icons are packing input read at their base level, so a mip chain would only cost
      // space.
      ImageFileWriter::write_dds(
        &destination.prepare_file(&relative_path)?,
        &options.source.view(x, y, w, h).to_image(),
        options.dds_compression_format,
        DdsMipmaps::Disabled,
      )?;

      Ok(true)
    }
  }
}
