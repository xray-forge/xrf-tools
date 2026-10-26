use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section};
use xrf_utils::{vector_from_string, vector_to_string};

#[cfg(test)]
use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
use crate::data::alife::inherited::alife_object_dynamic::AlifeObjectDynamic;

/// A squad's server object, `CSE_ALifeOnlineOfflineGroup`: the alife object, then a `u32` count of its members' object
/// ids. Its update packet is empty.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectOnlineOfflineGroup {
  pub base: AlifeObjectDynamic,
  /// The members' object ids, `m_members`, which the game assigns when it creates them.
  pub members: Vec<u16>,
}

impl ChunkReadWrite for AlifeObjectOnlineOfflineGroup {
  /// Read the group from the chunk.
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      base: reader.read_xr::<T, _>()?,
      members: reader.read_u16_vector::<T>()?,
    })
  }

  /// Write the group into the chunk.
  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_xr::<T, _>(&self.base)?;
    writer.write_u16_vector::<T>(&self.members)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectOnlineOfflineGroup {
  /// Import the group from ltx config section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeObjectDynamic::import(section_name, ltx)?,
      members: vector_from_string(&section.read_field::<String>("online_offline_group.members")?)?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("online_offline_group.members", vector_to_string(&self.members));

    Ok(())
  }
}

#[cfg(test)]
impl AlifeObjectOnlineOfflineGroup {
  /// A group of `members` at no particular vertex.
  pub fn new_mock(members: Vec<u16>) -> Self {
    Self {
      base: AlifeObjectDynamic {
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
      },
      members,
    }
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::{Ltx, LtxImportExport};

  use crate::data::alife::inherited::alife_object_online_offline_group::AlifeObjectOnlineOfflineGroup;

  /// The alife object's 27 bytes, then the count and two bytes a member.
  #[test]
  fn test_read_write() -> XrfResult {
    let original: AlifeObjectOnlineOfflineGroup = AlifeObjectOnlineOfflineGroup::new_mock(vec![6511, 6523]);
    let mut writer: ChunkWriter = ChunkWriter::new();

    original.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(writer.buffer.len(), 27 + 4 + 4);

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&writer.buffer)?;

    assert_eq!(
      AlifeObjectOnlineOfflineGroup::read::<XRayByteOrder, _>(&mut reader)?,
      original
    );
    reader.assert_read("Expect the group to be read whole")?;

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    for members in [vec![], vec![6511, 6523]] {
      let original: AlifeObjectOnlineOfflineGroup = AlifeObjectOnlineOfflineGroup::new_mock(members);
      let mut ltx: Ltx = Ltx::new();

      original.export("data", &mut ltx)?;

      assert_eq!(AlifeObjectOnlineOfflineGroup::import("data", &ltx)?, original);
    }

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectOnlineOfflineGroup = AlifeObjectOnlineOfflineGroup::new_mock(vec![6511]);

    assert_eq!(
      serde_json::from_str::<AlifeObjectOnlineOfflineGroup>(&serde_json::to_string(&original)?)?,
      original
    );

    Ok(())
  }
}
