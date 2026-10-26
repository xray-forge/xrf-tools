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
pub struct AlifeObjectItemPda {
  pub base: AlifeObjectItem,
  pub owner: u16,
  pub character: String,
  pub info_portion: String,
}

impl AlifePacketsReadWrite for AlifeObjectItemPda {
  /// Read pda object data from both packets; its own fields are all in the spawn packet.
  fn read<T: ByteOrder, D: ChunkDataSource>(
    spawn: &mut ChunkReader<D>,
    update: &mut ChunkReader<D>,
  ) -> XrfResult<Self> {
    Ok(Self {
      base: AlifeObjectItem::read::<T, _>(spawn, update)?,
      owner: spawn.read_u16::<T>()?,
      character: spawn.read_w1251_string()?,
      info_portion: spawn.read_w1251_string()?,
    })
  }

  /// Write pda object data into both packets.
  fn write<T: ByteOrder>(&self, spawn: &mut ChunkWriter, update: &mut ChunkWriter) -> XrfResult {
    self.base.write::<T>(spawn, update)?;
    spawn.write_u16::<T>(self.owner)?;
    spawn.write_w1251_string(&self.character)?;
    spawn.write_w1251_string(&self.info_portion)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectItemPda {
  /// Import pda object data from ltx config section.
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
      owner: section.read_field("item_pda.owner")?,
      character: section.read_field("item_pda.character")?,
      info_portion: section.read_field("item_pda.info_portion")?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("item_pda.owner", self.owner.to_string())
      .set("item_pda.character", &self.character)
      .set("item_pda.info_portion", &self.info_portion);

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
  use crate::data::alife::inherited::alife_object_item_pda::AlifeObjectItemPda;

  fn new_object() -> AlifeObjectItemPda {
    AlifeObjectItemPda {
      base: AlifeObjectItem::new_mock(Vector3d::new(-3.0, 0.5, 8.0)),
      owner: 3511,
      character: String::from("character"),
      info_portion: String::from("info-portion"),
    }
  }

  #[test]
  fn test_read_write() -> XrfResult {
    let original: AlifeObjectItemPda = new_object();

    assert_eq!(original.read_back::<XRayByteOrder>()?, (77, 54, original));

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let original: AlifeObjectItemPda = new_object();
    let mut ltx: Ltx = Ltx::new();

    original.export("data", &mut ltx)?;

    assert_eq!(AlifeObjectItemPda::import("data", &ltx)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectItemPda = new_object();

    assert_eq!(
      serde_json::from_str::<AlifeObjectItemPda>(&serde_json::to_string(&original)?)?,
      original
    );

    Ok(())
  }
}
