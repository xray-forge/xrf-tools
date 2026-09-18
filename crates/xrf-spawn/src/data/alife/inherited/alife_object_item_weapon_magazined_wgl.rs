use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_ltx::Ltx;
use xrf_ltx::LtxImportExport;

use crate::data::alife::inherited::alife_object_item_weapon_magazined::AlifeObjectItemWeaponMagazined;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectItemWeaponMagazinedWgl {
  pub base: AlifeObjectItemWeaponMagazined,
}

impl ChunkReadWrite for AlifeObjectItemWeaponMagazinedWgl {
  /// Read magazined weapon with launcher from the chunk.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      base: reader.read_xr::<T, _>()?,
    })
  }

  /// Write magazined weapon with launcher data into the writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.base)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectItemWeaponMagazinedWgl {
  /// Read magazined weapon with launcher from ltx config section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    Ok(Self {
      base: AlifeObjectItemWeaponMagazined::import(section_name, ltx)?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::utils::{
    build_relative_test_sample_file_path, open_generated_test_resource_as_slice,
    overwrite_generated_test_resource_as_file,
  };

  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
  use crate::data::alife::inherited::alife_object_item::AlifeObjectItem;
  use crate::data::alife::inherited::alife_object_item_weapon::AlifeObjectItemWeapon;
  use crate::data::alife::inherited::alife_object_item_weapon_magazined::AlifeObjectItemWeaponMagazined;
  use crate::data::alife::inherited::alife_object_item_weapon_magazined_wgl::AlifeObjectItemWeaponMagazinedWgl;

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: AlifeObjectItemWeaponMagazinedWgl = AlifeObjectItemWeaponMagazinedWgl {
      base: AlifeObjectItemWeaponMagazined {
        base: AlifeObjectItemWeapon {
          base: AlifeObjectItem {
            base: AlifeObjectDynamicVisual {
              base: AlifeObjectAbstract {
                game_vertex_id: 2413,
                distance: 53.1213,
                direct_control: 3643,
                level_vertex_id: 25313,
                flags: 32,
                custom_data: String::from("custom-data"),
                story_id: 3486,
                spawn_story_id: 37663,
              },
              visual_name: String::from("visual-name"),
              visual_flags: 36,
            },
            condition: 1.0,
            upgrades_count: 0,
          },
          ammo_current: 20,
          ammo_elapsed: 10,
          weapon_state: 1,
          addon_flags: 1,
          ammo_type: 1,
          elapsed_grenades: 0,
        },
      },
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 67);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 67);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 67 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(
      AlifeObjectItemWeaponMagazinedWgl::read::<XRayByteOrder, _>(&mut reader)?,
      original
    );

    Ok(())
  }
}
