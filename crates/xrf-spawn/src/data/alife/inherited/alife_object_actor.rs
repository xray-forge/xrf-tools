use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, LtxImportExport, Section};
use xrf_math::Vector3d;

use crate::data::alife::alife_packets_read_write::AlifePacketsReadWrite;
use crate::data::alife::inherited::alife_object_creature::AlifeObjectCreature;
use crate::data::alife::inherited::alife_object_skeleton::AlifeObjectSkeleton;
use crate::data::alife::inherited::alife_object_trader_abstract::AlifeObjectTraderAbstract;
use crate::data::physics::actor_physics_state::ActorPhysicsState;
use crate::data::physics::scaled_direction::ScaledDirection;

/// The actor, `CSE_ALifeCreatureActor`: a creature and a trader with a skeleton. Its update packet follows the
/// creature's fields with its own movement and the body it saved; the trader and the skeleton add none.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlifeObjectActor {
  pub base: AlifeObjectCreature,
  pub trader: AlifeObjectTraderAbstract,
  pub skeleton: AlifeObjectSkeleton,
  pub holder_id: u16,
  /// The movement state flags, `mstate`.
  pub update_movement_state: u16,
  pub update_acceleration: ScaledDirection,
  pub update_velocity: ScaledDirection,
  pub update_radiation: f32,
  /// The weapon slot in hand, `weapon`.
  pub update_weapon: u8,
  pub update_body: ActorPhysicsState,
}

impl AlifeObjectActor {
  /// Place the actor at `position` facing `direction`, carrying the body it saved along with it.
  pub fn move_to(&mut self, position: &Vector3d, direction: &Vector3d) {
    self.update_body.translate(&(position - &self.base.update_position));
    self.base.move_to(position, direction);
  }
}

impl AlifePacketsReadWrite for AlifeObjectActor {
  /// Read the actor from both packets.
  fn read<T: ByteOrder, D: ChunkDataSource>(
    spawn: &mut ChunkReader<D>,
    update: &mut ChunkReader<D>,
  ) -> XrfResult<Self> {
    Ok(Self {
      base: AlifeObjectCreature::read::<T, _>(spawn, update)?,
      trader: spawn.read_xr::<T, _>()?,
      skeleton: spawn.read_xr::<T, _>()?,
      holder_id: spawn.read_u16::<T>()?,
      update_movement_state: update.read_u16::<T>()?,
      update_acceleration: update.read_xr::<T, _>()?,
      update_velocity: update.read_xr::<T, _>()?,
      update_radiation: update.read_f32::<T>()?,
      update_weapon: update.read_u8()?,
      update_body: ActorPhysicsState::read::<T, _>(update)?,
    })
  }

  /// Write the actor into both packets.
  fn write<T: ByteOrder>(&self, spawn: &mut ChunkWriter, update: &mut ChunkWriter) -> XrfResult {
    self.base.write::<T>(spawn, update)?;

    spawn.write_xr::<T, _>(&self.trader)?;
    spawn.write_xr::<T, _>(&self.skeleton)?;
    spawn.write_u16::<T>(self.holder_id)?;

    update.write_u16::<T>(self.update_movement_state)?;
    update.write_xr::<T, _>(&self.update_acceleration)?;
    update.write_xr::<T, _>(&self.update_velocity)?;
    update.write_f32::<T>(self.update_radiation)?;
    update.write_u8(self.update_weapon)?;

    self.update_body.write::<T>(update)
  }
}

impl LtxImportExport for AlifeObjectActor {
  /// Import actor data from ltx config section.
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "ALife object '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    Ok(Self {
      base: AlifeObjectCreature::import(section_name, ltx)?,
      trader: AlifeObjectTraderAbstract::import(section_name, ltx)?,
      skeleton: AlifeObjectSkeleton::import(section_name, ltx)?,
      holder_id: section.read_field("actor.holder_id")?,
      update_movement_state: section.read_field("actor.update_movement_state")?,
      update_acceleration: section.read_field("actor.update_acceleration")?,
      update_velocity: section.read_field("actor.update_velocity")?,
      update_radiation: section.read_field("actor.update_radiation")?,
      update_weapon: section.read_field("actor.update_weapon")?,
      update_body: ActorPhysicsState::import(section, "actor.update_body")?,
    })
  }

  /// Export object data into ltx file.
  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    self.base.export(section_name, ltx)?;
    self.trader.export(section_name, ltx)?;
    self.skeleton.export(section_name, ltx)?;

    ltx
      .with_section(section_name)
      .set("actor.holder_id", self.holder_id.to_string())
      .set("actor.update_movement_state", self.update_movement_state.to_string())
      .set("actor.update_acceleration", self.update_acceleration.to_string())
      .set("actor.update_velocity", self.update_velocity.to_string())
      .set("actor.update_radiation", self.update_radiation.to_string())
      .set("actor.update_weapon", self.update_weapon.to_string());

    self.update_body.export(section_name, "actor.update_body", ltx);

    Ok(())
  }
}

#[cfg(test)]
impl AlifeObjectActor {
  /// An actor at `position` as a new game places one, with no body saved.
  pub fn new_mock(position: Vector3d) -> Self {
    Self {
      base: AlifeObjectCreature::new_mock(position),
      trader: AlifeObjectTraderAbstract {
        money: 5000,
        specific_character: String::from("actor"),
        trader_flags: 0,
        character_profile: String::from("actor"),
        community_index: 0,
        rank: 0,
        reputation: 0,
        character_name: String::new(),
        dead_body_can_take: 1,
        dead_body_closed: 0,
      },
      skeleton: AlifeObjectSkeleton {
        name: String::new(),
        flags: 0,
        source_id: u16::MAX,
      },
      holder_id: u16::MAX,
      update_movement_state: 0,
      update_acceleration: ScaledDirection {
        direction: 0,
        magnitude: 0.0,
      },
      update_velocity: ScaledDirection {
        direction: 0,
        magnitude: 0.0,
      },
      update_radiation: 0.0,
      update_weapon: 0xd7,
      update_body: ActorPhysicsState::Absent,
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
  use crate::data::alife::inherited::alife_object_actor::AlifeObjectActor;
  use crate::data::physics::actor_physics_state::ActorPhysicsState;
  use crate::data::physics::raw_float::RawFloat;

  fn new_alive_body(position: [f32; 3]) -> ActorPhysicsState {
    ActorPhysicsState::Alive {
      enabled: 1,
      angular_velocity: [RawFloat::from(0.0); 3],
      linear_velocity: [0.0, -1.0, 0.0].map(RawFloat::from),
      force: [RawFloat(u32::MAX); 3],
      torque: [RawFloat::from(0.0); 3],
      position: position.map(RawFloat::from),
      rotation: [0.0, 0.0, 0.0, 1.0].map(RawFloat::from),
    }
  }

  /// The creature's 40 bytes, then movement, two scaled directions, radiation, the weapon and the body's count: the
  /// 61-byte update packet a new game's actor carries.
  #[test]
  fn test_read_write() -> XrfResult {
    let original: AlifeObjectActor = AlifeObjectActor::new_mock(Vector3d::new(256.24, 19.81, 550.82));
    let (_, update_size, read) = original.read_back::<XRayByteOrder>()?;

    assert_eq!((update_size, read), (61, original));

    Ok(())
  }

  /// The enabled byte, five vectors and a quaternion follow the count.
  #[test]
  fn reads_and_writes_a_living_body() -> XrfResult {
    let mut original: AlifeObjectActor = AlifeObjectActor::new_mock(Vector3d::ZERO);

    original.update_body = new_alive_body([1.0, 2.0, 3.0]);

    let (_, update_size, read) = original.read_back::<XRayByteOrder>()?;

    assert_eq!((update_size, read), (61 + 1 + 5 * 12 + 16, original));

    Ok(())
  }

  /// A move carries a saved body by as far as the actor goes, so it stays where the actor stands.
  #[test]
  fn moves_its_body_along_with_it() {
    let mut actor: AlifeObjectActor = AlifeObjectActor::new_mock(Vector3d::new(10.0, 0.0, 10.0));

    actor.update_body = new_alive_body([10.0, 1.0, 10.0]);
    actor.move_to(&Vector3d::new(-5.0, 2.0, 40.0), &Vector3d::new(0.0, 3.0, 0.0));

    assert_eq!(actor.base.update_position, Vector3d::new(-5.0, 2.0, 40.0));
    assert_eq!(actor.base.update_torso_yaw, 3.0);
    // Only the body's position moves; its uninitialised force keeps its bits.
    assert_eq!(actor.update_body, new_alive_body([-5.0, 3.0, 40.0]));
  }

  #[test]
  fn test_import_export() -> XrfResult {
    let mut original: AlifeObjectActor = AlifeObjectActor::new_mock(Vector3d::new(1.0, 2.0, 3.0));
    let mut ltx: Ltx = Ltx::new();

    original.update_body = new_alive_body([1.0, 2.0, 3.0]);
    original.export("data", &mut ltx)?;

    assert_eq!(AlifeObjectActor::import("data", &ltx)?, original);

    Ok(())
  }

  #[test]
  fn test_serialize_deserialize() -> XrfResult {
    let mut original: AlifeObjectActor = AlifeObjectActor::new_mock(Vector3d::new(1.0, 2.0, 3.0));

    original.update_body = new_alive_body([1.0, 2.0, 3.0]);

    assert_eq!(
      serde_json::from_str::<AlifeObjectActor>(&serde_json::to_string(&original)?)?,
      original
    );

    Ok(())
  }
}
