use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section};

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
use crate::data::alife::inherited::alife_object_item::AlifeObjectItem;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectItemAmmo {
  pub base: AlifeObjectItem,
  pub ammo_left: u16,
  /// The rounds left as the update packet stores them, which the engine reads over `ammo_left`.
  pub update_ammo_left: u16,
}

impl AlifePacketsReadWrite for AlifeObjectItemAmmo {
  /// Read ammo from both packets, each carrying the rounds left.
  fn read<T: ByteOrder, D: ChunkDataSource>(
    spawn: &mut ChunkReader<D>,
    update: &mut ChunkReader<D>,
  ) -> XrfResult<Self> {
    Ok(Self {
      base: AlifeObjectItem::read::<T, _>(spawn, update)?,
      ammo_left: spawn.read_u16::<T>()?,
      update_ammo_left: update.read_u16::<T>()?,
    })
  }

  /// Write ammo into both packets.
  fn write<T: ByteOrder>(&self, spawn: &mut ChunkWriter, update: &mut ChunkWriter) -> XrfResult {
    self.base.write::<T>(spawn, update)?;
    spawn.write_u16::<T>(self.ammo_left)?;
    update.write_u16::<T>(self.update_ammo_left)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectItemAmmo {
  /// Import ALife ammo item data from ltx config section.
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
      ammo_left: section.read_field("item_ammo.ammo_left")?,
      update_ammo_left: section.read_field("item_ammo.update_ammo_left")?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("item_ammo.ammo_left", self.ammo_left.to_string())
      .set("item_ammo.update_ammo_left", self.update_ammo_left.to_string());

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::XRayByteOrder;
  use xrf_error::XrfResult;
  use xrf_ltx::{Ltx, LtxImportExport};
  use xrf_math::Vector3d;

  use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
  use crate::data::alife::inherited::alife_object_item::AlifeObjectItem;
  use crate::data::alife::inherited::alife_object_item_ammo::AlifeObjectItemAmmo;

  fn new_object() -> AlifeObjectItemAmmo {
    AlifeObjectItemAmmo {
      base: AlifeObjectItem::new_mock(Vector3d::new(2.0, 1.0, -7.5)),
      ammo_left: 30,
      update_ammo_left: 60,
    }
  }

  /// The rounds left follow the physics state in the update packet as they follow the item in the spawn packet.
  #[test]
  fn test_read_write() -> XrfResult {
    let original: AlifeObjectItemAmmo = new_object();

    assert_eq!(original.read_back::<XRayByteOrder>()?, (54, 56, original));

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let original: AlifeObjectItemAmmo = new_object();
    let mut ltx: Ltx = Ltx::new();

    original.export("data", &mut ltx)?;

    assert_eq!(AlifeObjectItemAmmo::import("data", &ltx)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectItemAmmo = new_object();

    assert_eq!(
      serde_json::from_str::<AlifeObjectItemAmmo>(&serde_json::to_string(&original)?)?,
      original
    );

    Ok(())
  }
}
