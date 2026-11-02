use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_ltx::{Ltx, LtxImportExport};

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
use crate::data::alife::inherited::alife_object_item::AlifeObjectItem;

/// An explosive, `CSE_ALifeItemExplosive`, which adds nothing to either packet of the item it is.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectItemExplosive {
  pub base: AlifeObjectItem,
}

impl AlifePacketsReadWrite for AlifeObjectItemExplosive {
  /// Read an explosive from both packets.
  fn read<T: ByteOrder, D: ChunkDataSource>(
    spawn: &mut ChunkReader<D>,
    update: &mut ChunkReader<D>,
  ) -> XrfResult<Self> {
    Ok(Self {
      base: AlifeObjectItem::read::<T, _>(spawn, update)?,
    })
  }

  /// Write an explosive into both packets.
  fn write<T: ByteOrder>(&self, spawn: &mut ChunkWriter, update: &mut ChunkWriter) -> XrfResult {
    self.base.write::<T>(spawn, update)
  }
}

impl LtxImportExport for AlifeObjectItemExplosive {
  /// Import an explosive from ltx config section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    Ok(Self {
      base: AlifeObjectItem::import(section_name, ltx)?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)
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
  use crate::data::alife::inherited::alife_object_item_explosive::AlifeObjectItemExplosive;

  fn new_object() -> AlifeObjectItemExplosive {
    AlifeObjectItemExplosive {
      base: AlifeObjectItem::new_mock(Vector3d::new(5.0, -1.5, 12.25)),
    }
  }

  #[test]
  fn test_read_write() -> XrfResult {
    let original: AlifeObjectItemExplosive = new_object();

    assert_eq!(original.read_back::<XRayByteOrder>()?, (52, 54, original));

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let original: AlifeObjectItemExplosive = new_object();
    let mut ltx: Ltx = Ltx::new();

    original.export("data", &mut ltx)?;

    assert_eq!(AlifeObjectItemExplosive::import("data", &ltx)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectItemExplosive = new_object();

    assert_eq!(
      serde_json::from_str::<AlifeObjectItemExplosive>(&serde_json::to_string(&original)?)?,
      original
    );

    Ok(())
  }
}
