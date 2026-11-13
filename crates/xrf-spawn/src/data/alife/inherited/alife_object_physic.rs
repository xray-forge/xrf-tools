use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReader, ChunkFieldWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxImportExport, LtxRead, LtxSectionRead, LtxSectionWrite, LtxWrite};

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
use crate::data::alife::inherited::alife_object_skeleton::AlifeObjectSkeleton;
use crate::data::alife::spawn_layout::SpawnLayout;
use crate::data::alife::spawn_packet_header::SpawnPacketHeader;
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
  /// Nothing to set: the physic object leaves its physics state's count uninitialised, so no state stands for it.
  fn set_spawned_update(&mut self, _: &SpawnPacketHeader) {}

  /// Read a physic object from both packets.
  ///
  /// An empty update packet reads as no physics state, as `CSE_ALifeObjectPhysic::UPDATE_Read` reads it; it is
  /// written back as the zero byte the engine writes for one. The state closes the packet, so it reads as
  /// [`PhysicsNetState::read_closing`] reads one, a state the SDK packed to zero among them.
  fn read<T: ByteOrder, S: ChunkFieldReader, U: ChunkFieldReader>(
    spawn: &mut S,
    update: &mut U,
    _: SpawnLayout,
  ) -> XrfResult<Self> {
    Ok(Self {
      base: spawn.read_xr::<T, _>()?,
      skeleton: spawn.read_xr::<T, _>()?,
      physic_type: spawn.read_u32::<T>()?,
      mass: spawn.read_f32::<T>()?,
      fixed_bones: spawn.read_w1251_string()?,
      update_state: PhysicsNetState::read_closing::<T, _>(update)?,
    })
  }

  /// Write a physic object into both packets.
  fn write<T: ByteOrder, S: ChunkFieldWriter, U: ChunkFieldWriter>(
    &self,
    spawn: &mut S,
    update: &mut U,
    _: SpawnLayout,
  ) -> XrfResult {
    spawn.write_xr::<T, _>(&self.base)?;
    spawn.write_xr::<T, _>(&self.skeleton)?;
    spawn.write_u32::<T>(self.physic_type)?;
    spawn.write_f32::<T>(self.mass)?;
    spawn.write_w1251_string(&self.fixed_bones)?;

    PhysicsNetState::write_closing::<T, _>(update, self.update_state.as_ref())
  }

  /// Import ALife physic object data from ltx config section.
  fn import<L: LtxRead>(section_name: &str, ltx: &L, _: SpawnLayout) -> XrfResult<Self> {
    let section: &L::Section = ltx.section(section_name).ok_or_else(|| {
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
  fn export(&self, section_name: &str, ltx: &mut impl LtxWrite, _: SpawnLayout) -> XrfResult {
    self.base.export(section_name, ltx)?;
    self.skeleton.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("physic_type", self.physic_type)
      .set("mass", self.mass)
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
  use xrf_ltx::Ltx;
  use xrf_math::Vector3d;

  use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
  use crate::data::alife::inherited::alife_object_abstract::AlifeObjectAbstract;
  use crate::data::alife::inherited::alife_object_dynamic_visual::AlifeObjectDynamicVisual;
  use crate::data::alife::inherited::alife_object_physic::AlifeObjectPhysic;
  use crate::data::alife::inherited::alife_object_skeleton::AlifeObjectSkeleton;
  use crate::data::alife::spawn_layout::SpawnLayout;
  use crate::data::physics::physics_net_state::PhysicsNetState;
  use crate::data::physics::raw_float::RawFloat;

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

    assert_eq!(
      original.read_back::<XRayByteOrder>(SpawnLayout::CALL_OF_PRIPYAT)?,
      (88, 54, original)
    );

    Ok(())
  }

  #[test]
  fn reads_and_writes_an_object_without_a_physics_state() -> XrfResult {
    let mut original: AlifeObjectPhysic = new_object();

    original.update_state = None;

    assert_eq!(
      original.read_back::<XRayByteOrder>(SpawnLayout::CALL_OF_PRIPYAT)?,
      (88, 1, original)
    );

    Ok(())
  }

  /// `UPDATE_Read` returns at once on an empty packet, leaving the object without a physics state.
  #[test]
  fn reads_an_empty_update_packet_as_no_physics_state() -> XrfResult {
    let mut expected: AlifeObjectPhysic = new_object();
    let mut spawn: xrf_chunk::ChunkWriter = xrf_chunk::ChunkWriter::new();
    let mut update: xrf_chunk::ChunkWriter = xrf_chunk::ChunkWriter::new();

    expected.update_state = None;
    expected.write::<XRayByteOrder, _, _>(&mut spawn, &mut update, SpawnLayout::CALL_OF_PRIPYAT)?;

    let mut spawn_reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&spawn.buffer)?;
    // The packet's length and `M_UPDATE` action, which `AlifeObject` reads, leaving nothing behind them.
    let mut update_reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[2, 0, 0, 0])?;

    update_reader.read_u32::<XRayByteOrder>()?;

    assert_eq!(
      AlifeObjectPhysic::read::<XRayByteOrder, _, _>(
        &mut spawn_reader,
        &mut update_reader,
        SpawnLayout::CALL_OF_PRIPYAT
      )?,
      expected
    );

    Ok(())
  }

  /// The SDK's 2009 factory writes the state of an object of no items and both velocities set behind a zero byte; it
  /// reads as that state and writes back byte for byte.
  #[test]
  fn reads_a_state_the_sdk_packed_to_zero() -> XrfResult {
    let mut original: AlifeObjectPhysic = new_object();
    let mut state: PhysicsNetState = PhysicsNetState::new_mock(Vector3d::new(1.0, 2.0, 3.0));

    state.item_count = 0;
    state.enabled = false;
    state.angular_velocity = Some([0.5, 0.0, 0.0].map(RawFloat::from));
    state.linear_velocity = Some([0.0, -1.0, 0.0].map(RawFloat::from));
    original.update_state = Some(state);

    assert_eq!(
      original.read_back::<XRayByteOrder>(SpawnLayout::CALL_OF_PRIPYAT)?,
      (88, 1 + 36 + 16 + 24 + 1, original)
    );

    Ok(())
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let original: AlifeObjectPhysic = new_object();
    let mut ltx: Ltx = Ltx::new();

    original.export("data", &mut ltx, SpawnLayout::CALL_OF_PRIPYAT)?;

    assert_eq!(
      AlifeObjectPhysic::import("data", &ltx, SpawnLayout::CALL_OF_PRIPYAT)?,
      original
    );

    Ok(())
  }
}
