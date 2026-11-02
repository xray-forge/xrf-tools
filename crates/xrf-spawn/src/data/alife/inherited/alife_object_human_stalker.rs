use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section};
use xrf_math::Vector3d;

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
use crate::data::alife::inherited::alife_object_human_abstract::AlifeObjectHumanAbstract;
use crate::data::alife::inherited::alife_object_skeleton::AlifeObjectSkeleton;

/// A stalker, `CSE_ALifeHumanStalker`: a human with a physics skeleton, whose update packet closes with the dialog it
/// starts a conversation with.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectHumanStalker {
  pub base: AlifeObjectHumanAbstract,
  pub skeleton: AlifeObjectSkeleton,
  /// `m_start_dialog`, which only the update packet stores; empty for the default.
  pub update_start_dialog: String,
}

impl AlifeObjectHumanStalker {
  /// Place the stalker at `position`, facing `direction`.
  pub fn move_to(&mut self, position: &Vector3d, direction: &Vector3d) {
    self.base.move_to(position, direction);
  }
}

impl AlifePacketsReadWrite for AlifeObjectHumanStalker {
  /// Read the stalker from both packets.
  fn read<T: ByteOrder, D: ChunkDataSource>(
    spawn: &mut ChunkReader<D>,
    update: &mut ChunkReader<D>,
  ) -> XrfResult<Self> {
    Ok(Self {
      base: AlifeObjectHumanAbstract::read::<T, _>(spawn, update)?,
      skeleton: spawn.read_xr::<T, _>()?,
      update_start_dialog: update.read_w1251_string()?,
    })
  }

  /// Write the stalker into both packets.
  fn write<T: ByteOrder>(&self, spawn: &mut ChunkWriter, update: &mut ChunkWriter) -> XrfResult {
    self.base.write::<T>(spawn, update)?;

    spawn.write_xr::<T, _>(&self.skeleton)?;
    update.write_w1251_string(&self.update_start_dialog)?;

    Ok(())
  }
}

impl LtxImportExport for AlifeObjectHumanStalker {
  /// Import stalker data from ltx config section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeObjectHumanAbstract::import(section_name, ltx)?,
      skeleton: AlifeObjectSkeleton::import(section_name, ltx)?,
      update_start_dialog: section.read_field("stalker.update_start_dialog")?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;
    self.skeleton.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("stalker.update_start_dialog", &self.update_start_dialog);

    Ok(())
  }
}

#[cfg(test)]
impl AlifeObjectHumanStalker {
  /// A stalker at `position` as the SDK compiles one, with no skeleton state and the default dialog.
  pub fn new_mock(position: Vector3d) -> Self {
    Self {
      base: AlifeObjectHumanAbstract::new_mock(position),
      skeleton: AlifeObjectSkeleton {
        name: String::new(),
        flags: 0,
        source_id: u16::MAX,
      },
      update_start_dialog: String::new(),
    }
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::XRayByteOrder;
  use xrf_error::XrfResult;
  use xrf_ltx::{Ltx, LtxImportExport};
  use xrf_math::Vector3d;

  use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
  use crate::data::alife::inherited::alife_object_human_stalker::AlifeObjectHumanStalker;

  fn new_object() -> AlifeObjectHumanStalker {
    let mut object: AlifeObjectHumanStalker = AlifeObjectHumanStalker::new_mock(Vector3d::new(1.0, 2.0, 3.0));

    object.update_start_dialog = String::from("zat_b30_owl_stalker_trader_start");

    object
  }

  /// The skeleton closes the spawn packet and the start dialog the update packet.
  #[test]
  fn test_read_write() -> XrfResult {
    let original: AlifeObjectHumanStalker = new_object();
    let (human_spawn_size, human_update_size, _) = original.base.read_back::<XRayByteOrder>()?;

    assert_eq!(
      original.read_back::<XRayByteOrder>()?,
      (
        human_spawn_size + 1 + 1 + 2,
        human_update_size + original.update_start_dialog.len() + 1,
        original
      )
    );

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let original: AlifeObjectHumanStalker = new_object();
    let mut ltx: Ltx = Ltx::new();

    original.export("data", &mut ltx)?;

    assert_eq!(AlifeObjectHumanStalker::import("data", &ltx)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let original: AlifeObjectHumanStalker = new_object();

    assert_eq!(
      serde_json::from_str::<AlifeObjectHumanStalker>(&serde_json::to_string(&original)?)?,
      original
    );

    Ok(())
  }
}
