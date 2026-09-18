use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::LtxImportExport;
use xrf_ltx::read_ltx_field;
use xrf_ltx::{Ltx, Section};

use crate::data::alife::inherited::alife_object_item::AlifeObjectItem;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectItemWeapon {
  pub base: AlifeObjectItem,
  pub ammo_current: u16,
  pub ammo_elapsed: u16,
  pub weapon_state: u8,
  pub addon_flags: u8,
  pub ammo_type: u8,
  pub elapsed_grenades: u8,
}

impl ChunkReadWrite for AlifeObjectItemWeapon {
  /// Read ALife item object data from the chunk.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      base: reader.read_xr::<T, _>()?,
      ammo_current: reader.read_u16::<T>()?,
      ammo_elapsed: reader.read_u16::<T>()?,
      weapon_state: reader.read_u8()?,
      addon_flags: reader.read_u8()?,
      ammo_type: reader.read_u8()?,
      elapsed_grenades: reader.read_u8()?,
    })
  }

  /// Write item data into the writer.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.base)?;
    writer.write_u16::<T>(self.ammo_current)?;
    writer.write_u16::<T>(self.ammo_elapsed)?;
    writer.write_u8(self.weapon_state)?;
    writer.write_u8(self.addon_flags)?;
    writer.write_u8(self.ammo_type)?;
    writer.write_u8(self.elapsed_grenades)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectItemWeapon {
  /// Import ALife weapon item object data from ltx config section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeObjectItem::import(section_name, ltx)?,
      ammo_current: read_ltx_field("item_weapon.ammo_current", section)?,
      ammo_elapsed: read_ltx_field("item_weapon.ammo_elapsed", section)?,
      weapon_state: read_ltx_field("item_weapon.weapon_state", section)?,
      addon_flags: read_ltx_field("item_weapon.addon_flags", section)?,
      ammo_type: read_ltx_field("item_weapon.ammo_type", section)?,
      elapsed_grenades: read_ltx_field("item_weapon.elapsed_grenades", section)?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("item_weapon.ammo_current", self.ammo_current.to_string())
      .set("item_weapon.ammo_elapsed", self.ammo_elapsed.to_string())
      .set("item_weapon.weapon_state", self.weapon_state.to_string())
      .set("item_weapon.addon_flags", self.addon_flags.to_string())
      .set("item_weapon.ammo_type", self.ammo_type.to_string())
      .set("item_weapon.elapsed_grenades", self.elapsed_grenades.to_string());

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::fs::File;
  use std::io::{Seek, SeekFrom, Write};

  use serde_json::to_string_pretty;
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;
  use xrf_ltx::LtxImportExport;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::file::read_file_as_string;
  use xrf_test_utils::utils::{
    build_absolute_generated_test_resource_path, build_relative_test_sample_file_path,
    open_generated_test_resource_as_slice, overwrite_generated_test_resource_as_file,
  };

  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
  use crate::data::alife::inherited::alife_object_item::AlifeObjectItem;
  use crate::data::alife::inherited::alife_object_item_weapon::AlifeObjectItemWeapon;

  #[test]
  fn test_read_write() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write.chunk");

    let original: AlifeObjectItemWeapon = AlifeObjectItemWeapon {
      base: AlifeObjectItem {
        base: AlifeObjectDynamicVisual {
          base: AlifeObjectAbstract {
            game_vertex_id: 1011,
            distance: 234.2511,
            direct_control: 623354,
            level_vertex_id: 455313,
            flags: 43,
            custom_data: String::from("custom_data"),
            story_id: 512,
            spawn_story_id: 31,
          },
          visual_name: String::from("asdfgh"),
          visual_flags: 33,
        },
        condition: 0.84,
        upgrades_count: 0,
      },
      ammo_current: 25,
      ammo_elapsed: 5,
      weapon_state: 1,
      addon_flags: 0,
      ammo_type: 1,
      elapsed_grenades: 0,
    };

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.bytes_written(), 62);

    let bytes_written: usize =
      writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    assert_eq!(bytes_written, 62);

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;

    assert_eq!(file.bytes_remaining(), 62 + 8);

    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(AlifeObjectItemWeapon::read::<XRayByteOrder, _>(&mut reader)?, original);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let ltx_filename: String = build_relative_test_sample_file_path(file!(), "import_export.ltx");
    let mut ltx: Ltx = Ltx::new();

    let original: AlifeObjectItemWeapon = AlifeObjectItemWeapon {
      base: AlifeObjectItem {
        base: AlifeObjectDynamicVisual {
          base: AlifeObjectAbstract {
            game_vertex_id: 67,
            distance: 3.2511,
            direct_control: 4,
            level_vertex_id: 5,
            flags: 6,
            custom_data: String::from("custom_data"),
            story_id: 12,
            spawn_story_id: 16,
          },
          visual_name: String::from("asdfgh"),
          visual_flags: 25,
        },
        condition: 0.387,
        upgrades_count: 0,
      },
      ammo_current: 35,
      ammo_elapsed: 5,
      weapon_state: 2,
      addon_flags: 1,
      ammo_type: 4,
      elapsed_grenades: 4,
    };

    original.export("data", &mut ltx)?;

    ltx.write_to(&mut overwrite_generated_test_resource_as_file(&ltx_filename)?)?;

    let source: Ltx = Ltx::read_from_path(build_absolute_generated_test_resource_path(&ltx_filename))?;

    assert_eq!(AlifeObjectItemWeapon::import("data", &source)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectItemWeapon = AlifeObjectItemWeapon {
      base: AlifeObjectItem {
        base: AlifeObjectDynamicVisual {
          base: AlifeObjectAbstract {
            game_vertex_id: 63,
            distance: 3745.2511,
            direct_control: 2135,
            level_vertex_id: 2543,
            flags: 42,
            custom_data: String::from("custom_data"),
            story_id: 152,
            spawn_story_id: 1428,
          },
          visual_name: String::from("asdfgh"),
          visual_flags: 40,
        },
        condition: 0.4,
        upgrades_count: 2,
      },
      ammo_current: 21,
      ammo_elapsed: 9,
      weapon_state: 2,
      addon_flags: 12,
      ammo_type: 3,
      elapsed_grenades: 1,
    };

    let mut file: File = overwrite_generated_test_resource_as_file(&build_relative_test_sample_file_path(
      file!(),
      "serialize_deserialize.json",
    ))?;

    file.write_all(to_string_pretty(&original)?.as_bytes())?;
    file.seek(SeekFrom::Start(0))?;

    let serialized: String = read_file_as_string(&mut file)?;

    assert_eq!(serialized.to_string(), serialized);

    assert_eq!(serde_json::from_str::<AlifeObjectItemWeapon>(&serialized)?, original);

    Ok(())
  }
}
