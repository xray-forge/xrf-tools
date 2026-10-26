use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section};
#[cfg(test)]
use xrf_math::Vector3d;

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
use crate::data::alife::inherited::alife_object_actor::AlifeObjectActor;
use crate::data::generic::script_save_marker::ScriptSaveMarker;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeActor {
  pub base: AlifeObjectActor,
  /// The script class's own field, `se_actor` in vanilla.
  pub start_position_filled: u8,
}

impl AlifeActor {
  /// The script class the trailing field belongs to, named in a refused save marker.
  const SCRIPT_CLASS: &'static str = "se_actor";
}

#[cfg(test)]
impl AlifeActor {
  /// An actor at `position` as a new game places one, with no body saved.
  pub fn new_mock(position: Vector3d) -> Self {
    Self {
      base: AlifeObjectActor::new_mock(position),
      start_position_filled: 0,
    }
  }
}

impl AlifePacketsReadWrite for AlifeActor {
  /// Read actor data from both packets; the script class adds to the spawn packet alone.
  fn read<T: ByteOrder, D: ChunkDataSource>(
    spawn: &mut ChunkReader<D>,
    update: &mut ChunkReader<D>,
  ) -> XrfResult<Self> {
    Ok(Self {
      base: AlifeObjectActor::read::<T, _>(spawn, update)?,
      start_position_filled: ScriptSaveMarker::read::<T, _, _>(spawn, Self::SCRIPT_CLASS, |reader| {
        Ok(reader.read_u8()?)
      })?,
    })
  }

  /// Write actor data into both packets.
  fn write<T: ByteOrder>(&self, spawn: &mut ChunkWriter, update: &mut ChunkWriter) -> XrfResult {
    self.base.write::<T>(spawn, update)?;

    ScriptSaveMarker::write::<T>(spawn, Self::SCRIPT_CLASS, |writer| {
      writer.write_u8(self.start_position_filled)?;

      Ok(())
    })
  }
}

impl LtxImportExport for AlifeActor {
  /// Import actor data from ltx file section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeObjectActor::import(section_name, ltx)?,
      start_position_filled: section.read_field("actor.start_position_filled")?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("actor.start_position_filled", self.start_position_filled.to_string());

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
  use crate::data::alife::inherited::alife_actor::AlifeActor;

  fn new_object() -> AlifeActor {
    let mut actor: AlifeActor = AlifeActor::new_mock(Vector3d::new(256.24, 19.81, 550.82));

    actor.start_position_filled = 1;

    actor
  }

  /// The script class's byte and the marker counting it close the spawn packet; it adds nothing to the update.
  #[test]
  fn test_read_write() -> XrfResult {
    let original: AlifeActor = new_object();
    let (_, update_size, read) = original.read_back::<XRayByteOrder>()?;

    assert_eq!((update_size, read), (61, original));

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let original: AlifeActor = new_object();
    let mut ltx: Ltx = Ltx::new();

    original.export("data", &mut ltx)?;

    assert_eq!(AlifeActor::import("data", &ltx)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeActor = new_object();

    assert_eq!(
      serde_json::from_str::<AlifeActor>(&serde_json::to_string(&original)?)?,
      original
    );

    Ok(())
  }
}
