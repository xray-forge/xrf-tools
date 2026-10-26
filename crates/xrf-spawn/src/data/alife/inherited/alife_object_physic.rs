use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section};

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
use crate::data::alife::inherited::alife_object_skeleton::AlifeObjectSkeleton;
use crate::data::physics::physics_net_state::PhysicsNetState;

#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectPhysic {
  pub base: AlifeObjectDynamicVisual,
  pub skeleton: AlifeObjectSkeleton,
  pub physic_type: u32,
  pub mass: f32,
  pub fixed_bones: String,
  /// The physics state the update packet carries. `CPhysicObject::net_Import` drops it for a local object, so a single
  /// player game never applies it, which is why vanilla's are often left uninitialised.
  pub update_state: Option<PhysicsNetState>,
}

impl AlifePacketsReadWrite for AlifeObjectPhysic {
  /// Read a physic object from both packets.
  ///
  /// An empty update packet reads as no physics state, as `CSE_ALifeObjectPhysic::UPDATE_Read` reads it; it is
  /// written back as the zero byte the engine writes for one.
  fn read<T: ByteOrder, D: ChunkDataSource>(
    spawn: &mut ChunkReader<D>,
    update: &mut ChunkReader<D>,
  ) -> XrfResult<Self> {
    Ok(Self {
      base: spawn.read_xr::<T, _>()?,
      skeleton: spawn.read_xr::<T, _>()?,
      physic_type: spawn.read_u32::<T>()?,
      mass: spawn.read_f32::<T>()?,
      fixed_bones: spawn.read_w1251_string()?,
      update_state: if update.is_ended() {
        None
      } else {
        PhysicsNetState::read_optional::<T, _>(update)?
      },
    })
  }

  /// Write a physic object into both packets.
  fn write<T: ByteOrder>(&self, spawn: &mut ChunkWriter, update: &mut ChunkWriter) -> XrfResult {
    spawn.write_xr::<T, _>(&self.base)?;
    spawn.write_xr::<T, _>(&self.skeleton)?;
    spawn.write_u32::<T>(self.physic_type)?;
    spawn.write_f32::<T>(self.mass)?;
    spawn.write_w1251_string(&self.fixed_bones)?;

    PhysicsNetState::write_optional::<T>(self.update_state.as_ref(), update)
  }
}

impl LtxImportExport for AlifeObjectPhysic {
  /// Import ALife physic object data from ltx config section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeObjectDynamicVisual::import(section_name, ltx)?,
      skeleton: AlifeObjectSkeleton::import(section_name, ltx)?,
      physic_type: section.read_field("physic_type")?,
      mass: section.read_field("mass")?,
      fixed_bones: section.read_field("fixed_bones")?,
      update_state: PhysicsNetState::import_optional(section, "physic.update_state")?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;
    self.skeleton.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("physic_type", self.physic_type.to_string())
      .set("mass", self.mass.to_string())
      .set("fixed_bones", &self.fixed_bones);

    PhysicsNetState::export_optional(self.update_state.as_ref(), section_name, "physic.update_state", ltx);

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use byteorder::ReadBytesExt;
  use xrf_chunk::{ChunkReader, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::{Ltx, LtxImportExport};
  use xrf_math::Vector3d;

  use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
  use crate::data::alife::inherited::alife_object_physic::AlifeObjectPhysic;
  use crate::data::alife::inherited::alife_object_skeleton::AlifeObjectSkeleton;
  use crate::data::physics::physics_net_state::PhysicsNetState;

  fn new_object() -> AlifeObjectPhysic {
    AlifeObjectPhysic {
      base: AlifeObjectDynamicVisual {
        base: AlifeObjectAbstract {
          game_vertex_id: 35794,
          distance: 25.23,
          direct_control: 1243,
          level_vertex_id: 34623,
          flags: 62,
          custom_data: String::from("custom-data"),
          story_id: 825679,
          spawn_story_id: 1452,
        },
        visual_name: String::from("visual-name"),
        visual_flags: 34,
      },
      skeleton: AlifeObjectSkeleton {
        name: String::from("skeleton-name"),
        flags: 0,
        source_id: 2153,
      },
      physic_type: 6,
      mass: 5.0,
      fixed_bones: String::from("fixed-bones"),
      update_state: Some(PhysicsNetState::new_mock(Vector3d::new(10.0, 0.5, -3.0))),
    }
  }

  #[test]
  fn test_read_write() -> XrfResult {
    let original: AlifeObjectPhysic = new_object();

    assert_eq!(original.read_back::<XRayByteOrder>()?, (88, 54, original));

    Ok(())
  }

  #[test]
  fn reads_and_writes_an_object_without_a_physics_state() -> XrfResult {
    let mut original: AlifeObjectPhysic = new_object();

    original.update_state = None;

    assert_eq!(original.read_back::<XRayByteOrder>()?, (88, 1, original));

    Ok(())
  }

  /// `UPDATE_Read` returns at once on an empty packet, leaving the object without a physics state.
  #[test]
  fn reads_an_empty_update_packet_as_no_physics_state() -> XrfResult {
    let mut expected: AlifeObjectPhysic = new_object();
    let mut spawn: xrf_chunk::ChunkWriter = xrf_chunk::ChunkWriter::new();
    let mut update: xrf_chunk::ChunkWriter = xrf_chunk::ChunkWriter::new();

    expected.update_state = None;
    expected.write::<XRayByteOrder>(&mut spawn, &mut update)?;

    let mut spawn_reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&spawn.buffer)?;
    // The packet's length and `M_UPDATE` action, which `AlifeObject` reads, leaving nothing behind them.
    let mut update_reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[2, 0, 0, 0])?;

    update_reader.read_u32::<XRayByteOrder>()?;

    assert_eq!(
      AlifeObjectPhysic::read::<XRayByteOrder, _>(&mut spawn_reader, &mut update_reader)?,
      expected
    );

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let original: AlifeObjectPhysic = new_object();
    let mut ltx: Ltx = Ltx::new();

    original.export("data", &mut ltx)?;

    assert_eq!(AlifeObjectPhysic::import("data", &ltx)?, original);

    Ok(())
  }
}
