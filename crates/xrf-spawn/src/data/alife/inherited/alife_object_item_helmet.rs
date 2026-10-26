use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section};

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
use crate::data::alife::inherited::alife_object_item::AlifeObjectItem;

/// A helmet, `CSE_ALifeItemHelmet`, whose update packet repeats its condition after the item's.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectItemHelmet {
  pub base: AlifeObjectItem,
  /// The condition as the update packet quantises it, which the engine reads over the spawn packet's.
  pub update_condition: u8,
}

impl AlifeObjectItemHelmet {
  /// The condition the engine reads from the update packet.
  pub fn get_update_condition(&self) -> f32 {
    AlifeObjectItem::dequantize_condition(self.update_condition)
  }
}

impl AlifePacketsReadWrite for AlifeObjectItemHelmet {
  /// Read a helmet from both packets.
  fn read<T: ByteOrder, D: ChunkDataSource>(
    spawn: &mut ChunkReader<D>,
    update: &mut ChunkReader<D>,
  ) -> XrfResult<Self> {
    Ok(Self {
      base: AlifeObjectItem::read::<T, _>(spawn, update)?,
      update_condition: update.read_u8()?,
    })
  }

  /// Write a helmet into both packets.
  fn write<T: ByteOrder>(&self, spawn: &mut ChunkWriter, update: &mut ChunkWriter) -> XrfResult {
    self.base.write::<T>(spawn, update)?;
    update.write_u8(self.update_condition)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectItemHelmet {
  /// Import a helmet from ltx config section.
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
      update_condition: section.read_field("item_helmet.update_condition")?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("item_helmet.update_condition", self.update_condition.to_string());

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
  use crate::data::alife::inherited::alife_object_item_helmet::AlifeObjectItemHelmet;

  fn new_object() -> AlifeObjectItemHelmet {
    AlifeObjectItemHelmet {
      base: AlifeObjectItem::new_mock(Vector3d::new(5.0, -1.5, 12.25)),
      update_condition: 255,
    }
  }

  /// The quantised condition follows the item's physics state, one byte past it.
  #[test]
  fn test_read_write() -> XrfResult {
    let original: AlifeObjectItemHelmet = new_object();

    assert_eq!(original.read_back::<XRayByteOrder>()?, (52, 55, original));

    Ok(())
  }

  /// With no physics state the item's zero byte comes first, then the condition: the two bytes such an item carries.
  #[test]
  fn reads_the_condition_after_an_absent_physics_state() -> XrfResult {
    let mut original: AlifeObjectItemHelmet = new_object();

    original.base.update_state = None;
    original.update_condition = 128;

    assert_eq!(original.read_back::<XRayByteOrder>()?, (52, 2, original.clone()));
    assert!((original.get_update_condition() - 0.502).abs() < 1e-3);

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let original: AlifeObjectItemHelmet = new_object();
    let mut ltx: Ltx = Ltx::new();

    original.export("data", &mut ltx)?;

    assert_eq!(AlifeObjectItemHelmet::import("data", &ltx)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectItemHelmet = new_object();

    assert_eq!(
      serde_json::from_str::<AlifeObjectItemHelmet>(&serde_json::to_string(&original)?)?,
      original
    );

    Ok(())
  }
}
