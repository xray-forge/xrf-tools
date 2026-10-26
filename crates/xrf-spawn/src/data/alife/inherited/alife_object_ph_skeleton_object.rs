use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_ltx::{Ltx, LtxImportExport};

use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
use crate::data::alife::inherited::alife_object_skeleton::AlifeObjectSkeleton;

/// A physics skeleton object, `CSE_ALifePHSkeletonObject`: a visual with a physics skeleton, such as a corpse left as
/// a prop. Its update packet is empty.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectPhSkeletonObject {
  pub base: AlifeObjectDynamicVisual,
  pub skeleton: AlifeObjectSkeleton,
}

impl ChunkReadWrite for AlifeObjectPhSkeletonObject {
  /// Read the skeleton object from the chunk.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      base: reader.read_xr::<T, _>()?,
      skeleton: reader.read_xr::<T, _>()?,
    })
  }

  /// Write the skeleton object into the chunk.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.base)?;
    writer.write_xr::<T, _>(&self.skeleton)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectPhSkeletonObject {
  /// Import the skeleton object from ltx config section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    Ok(Self {
      base: AlifeObjectDynamicVisual::import(section_name, ltx)?,
      skeleton: AlifeObjectSkeleton::import(section_name, ltx)?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;
    self.skeleton.export(section_name, ltx)?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::{Ltx, LtxImportExport};

  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
  use crate::data::alife::inherited::alife_object_ph_skeleton_object::AlifeObjectPhSkeletonObject;
  use crate::data::alife::inherited::alife_object_skeleton::AlifeObjectSkeleton;

  fn new_object() -> AlifeObjectPhSkeletonObject {
    AlifeObjectPhSkeletonObject {
      base: AlifeObjectDynamicVisual {
        base: AlifeObjectAbstract {
          game_vertex_id: 621,
          distance: 0.0,
          direct_control: 1,
          level_vertex_id: 52235,
          flags: 0xffff_ffbf,
          custom_data: String::new(),
          story_id: AlifeObjectAbstract::INVALID_STORY_ID,
          spawn_story_id: AlifeObjectAbstract::INVALID_STORY_ID,
        },
        visual_name: String::from("actors\\stalker_bandit\\stalker_bandit_3"),
        visual_flags: 0,
      },
      skeleton: AlifeObjectSkeleton {
        name: String::from("$editor"),
        flags: 0,
        source_id: u16::MAX,
      },
    }
  }

  /// The visual's 27 alife bytes, its name and flags, then the skeleton's animation, flags and source.
  #[test]
  fn test_read_write() -> XrfResult {
    let original: AlifeObjectPhSkeletonObject = new_object();
    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(
      writer.buffer.len(),
      27 + original.base.visual_name.len() + 1 + 1 + 8 + 1 + 2
    );

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&writer.buffer)?;

    assert_eq!(
      AlifeObjectPhSkeletonObject::read::<XRayByteOrder, _>(&mut reader)?,
      original
    );
    reader.assert_read("Expect the skeleton object to be read whole")?;

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let original: AlifeObjectPhSkeletonObject = new_object();
    let mut ltx: Ltx = Ltx::new();

    original.export("data", &mut ltx)?;

    assert_eq!(AlifeObjectPhSkeletonObject::import("data", &ltx)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectPhSkeletonObject = new_object();

    assert_eq!(
      serde_json::from_str::<AlifeObjectPhSkeletonObject>(&serde_json::to_string(&original)?)?,
      original
    );

    Ok(())
  }
}
