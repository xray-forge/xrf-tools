use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section};

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
use crate::data::alife::inherited::alife_object_item::AlifeObjectItem;

/// A torch, `CSE_ALifeItemTorch`, whose update packet closes with a byte of whether its light and night vision are on
/// and whether it is attached; its spawn packet is the item's.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectItemTorch {
  pub base: AlifeObjectItem,
  /// `eTorchActive`, `eNightVisionActive` and `eAttached` in its three low bits.
  pub update_flags: u8,
}

impl AlifeObjectItemTorch {
  /// `eTorchActive`: the light is on.
  pub const FLAG_ACTIVE: u8 = 1 << 0;
  /// `eNightVisionActive`: the night vision is on.
  pub const FLAG_NIGHT_VISION_ACTIVE: u8 = 1 << 1;
  /// `eAttached`: the torch sits on an owner's head.
  pub const FLAG_ATTACHED: u8 = 1 << 2;
}

impl AlifePacketsReadWrite for AlifeObjectItemTorch {
  /// Read a torch from both packets.
  fn read<T: ByteOrder, D: ChunkDataSource>(
    spawn: &mut ChunkReader<D>,
    update: &mut ChunkReader<D>,
  ) -> XrfResult<Self> {
    Ok(Self {
      base: AlifeObjectItem::read::<T, _>(spawn, update)?,
      update_flags: update.read_u8()?,
    })
  }

  /// Write a torch into both packets.
  fn write<T: ByteOrder>(&self, spawn: &mut ChunkWriter, update: &mut ChunkWriter) -> XrfResult {
    self.base.write::<T>(spawn, update)?;
    update.write_u8(self.update_flags)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectItemTorch {
  /// Import a torch from ltx config section.
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
      update_flags: section.read_field("item_torch.update_flags")?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("item_torch.update_flags", self.update_flags.to_string());

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
  use crate::data::alife::inherited::alife_object_item_torch::AlifeObjectItemTorch;

  fn new_object() -> AlifeObjectItemTorch {
    AlifeObjectItemTorch {
      base: AlifeObjectItem::new_mock(Vector3d::new(5.0, -1.5, 12.25)),
      update_flags: AlifeObjectItemTorch::FLAG_ACTIVE | AlifeObjectItemTorch::FLAG_ATTACHED,
    }
  }

  /// The flags byte follows the item's physics state.
  #[test]
  fn test_read_write() -> XrfResult {
    let original: AlifeObjectItemTorch = new_object();

    assert_eq!(original.read_back::<XRayByteOrder>()?, (52, 55, original));

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let original: AlifeObjectItemTorch = new_object();
    let mut ltx: Ltx = Ltx::new();

    original.export("data", &mut ltx)?;

    assert_eq!(AlifeObjectItemTorch::import("data", &ltx)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectItemTorch = new_object();

    assert_eq!(
      serde_json::from_str::<AlifeObjectItemTorch>(&serde_json::to_string(&original)?)?,
      original
    );

    Ok(())
  }
}
