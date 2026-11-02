use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section};

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
use crate::data::alife::inherited::alife_object_item::AlifeObjectItem;

/// A document, `CSE_ALifeItemDocument`: an item whose spawn packet closes with the info portion reading it gives; its
/// update packet is the item's.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectItemDocument {
  pub base: AlifeObjectItem,
  /// The info portion, `m_wDoc`.
  pub info_portion: String,
}

impl AlifePacketsReadWrite for AlifeObjectItemDocument {
  /// Read a document from both packets.
  fn read<T: ByteOrder, D: ChunkDataSource>(
    spawn: &mut ChunkReader<D>,
    update: &mut ChunkReader<D>,
  ) -> XrfResult<Self> {
    Ok(Self {
      base: AlifeObjectItem::read::<T, _>(spawn, update)?,
      info_portion: spawn.read_w1251_string()?,
    })
  }

  /// Write a document into both packets.
  fn write<T: ByteOrder>(&self, spawn: &mut ChunkWriter, update: &mut ChunkWriter) -> XrfResult {
    self.base.write::<T>(spawn, update)?;
    spawn.write_w1251_string(&self.info_portion)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectItemDocument {
  /// Import a document from ltx config section.
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
      info_portion: section.read_field("item_document.info_portion")?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("item_document.info_portion", &self.info_portion);

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
  use crate::data::alife::inherited::alife_object_item_document::AlifeObjectItemDocument;

  fn new_object() -> AlifeObjectItemDocument {
    AlifeObjectItemDocument {
      base: AlifeObjectItem::new_mock(Vector3d::new(5.0, -1.5, 12.25)),
      info_portion: String::from("zat_b12_documents_read"),
    }
  }

  /// The info portion closes the item's spawn fields; the update packet is the item's alone.
  #[test]
  fn test_read_write() -> XrfResult {
    let original: AlifeObjectItemDocument = new_object();

    assert_eq!(
      original.read_back::<XRayByteOrder>()?,
      (52 + original.info_portion.len() + 1, 54, original)
    );

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let original: AlifeObjectItemDocument = new_object();
    let mut ltx: Ltx = Ltx::new();

    original.export("data", &mut ltx)?;

    assert_eq!(AlifeObjectItemDocument::import("data", &ltx)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectItemDocument = new_object();

    assert_eq!(
      serde_json::from_str::<AlifeObjectItemDocument>(&serde_json::to_string(&original)?)?,
      original
    );

    Ok(())
  }
}
