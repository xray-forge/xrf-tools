use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReadWrite, ChunkFieldReader, ChunkFieldWriter};
use xrf_error::XrfResult;
use xrf_ltx::{LtxSectionRead, LtxSectionWrite, LtxWrite};

use crate::data::generic::script_absent_state::ScriptAbsentState;
use crate::data::generic::script_save_marker::ScriptSaveMarker;

/// An id generator of Clear Sky's scripts, `object_collection:save` (`gamedata-cs/scripts/object_collection.script`):
/// how many ids are given and the last one, then the freed and the given ids, closed by a save marker. A spawn holds a
/// generator that gave none, so the two id lists are read as empty and any other count is refused rather than read.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearSkyObjectCollection {
  /// `m_count`.
  pub count: u16,
  /// `m_last_id`.
  pub last_id: u16,
}

impl ClearSkyObjectCollection {
  const SCRIPT_CLASS: &'static str = "object_collection";

  /// Import the generator kept under `prefix`.
  pub fn import(section: &impl LtxSectionRead, prefix: &str) -> XrfResult<Self> {
    Ok(Self {
      count: section.read_field(&format!("{prefix}.count"))?,
      last_id: section.read_field(&format!("{prefix}.last_id"))?,
    })
  }

  /// Export the generator under `prefix`.
  pub fn export(&self, section_name: &str, prefix: &str, ltx: &mut impl LtxWrite) {
    ltx
      .with_section(section_name)
      .set(format!("{prefix}.count"), self.count)
      .set(format!("{prefix}.last_id"), self.last_id);
  }
}

impl ChunkFieldReadWrite for ClearSkyObjectCollection {
  fn read_fields<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Self> {
    ScriptSaveMarker::read::<T, _, _>(reader, Self::SCRIPT_CLASS, |reader| {
      let collection: Self = Self {
        count: reader.read_u16::<T>()?,
        last_id: reader.read_u16::<T>()?,
      };

      ScriptAbsentState::read_u16::<T, _>(reader, Self::SCRIPT_CLASS, "freed ids count", 0)?;
      ScriptAbsentState::read_u16::<T, _>(reader, Self::SCRIPT_CLASS, "given ids count", 0)?;

      Ok(collection)
    })
  }

  fn write_fields<T: ByteOrder, W: ChunkFieldWriter>(&self, writer: &mut W) -> XrfResult {
    ScriptSaveMarker::write::<T, _>(writer, Self::SCRIPT_CLASS, |writer| {
      writer.write_u16::<T>(self.count)?;
      writer.write_u16::<T>(self.last_id)?;
      // No freed and no given ids.
      writer.write_u16::<T>(0)?;
      writer.write_u16::<T>(0)
    })
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;

  use crate::data::alife::inherited::clear_sky::actor::clear_sky_object_collection::ClearSkyObjectCollection;

  #[test]
  fn reads_and_writes_a_generator_that_gave_none() -> XrfResult {
    let bytes: &[u8] = b"\0\0\0\0\0\0\0\0\x08\0";
    let mut writer: ChunkWriter = ChunkWriter::new();

    ClearSkyObjectCollection::default().write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.buffer, bytes);
    assert_eq!(
      ClearSkyObjectCollection::read::<XRayByteOrder, _>(&mut ChunkReader::<InMemoryChunkDataSource>::from_bytes(
        bytes
      )?)?,
      ClearSkyObjectCollection::default()
    );

    Ok(())
  }

  /// A given id would be followed by the list holding it, which is not read.
  #[test]
  fn refuses_given_ids() -> XrfResult {
    let error: String = ClearSkyObjectCollection::read::<XRayByteOrder, _>(
      &mut ChunkReader::<InMemoryChunkDataSource>::from_bytes(b"\x01\0\x01\0\0\0\x01\0\x01\0\x0a\0")?,
    )
    .expect_err("one given id")
    .to_string();

    assert!(error.contains("object_collection holds given ids count 1"), "{error}");

    Ok(())
  }
}
