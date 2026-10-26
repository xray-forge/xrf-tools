use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, Section};
use xrf_math::Vector3d;
use xrf_utils::vector_to_string;

use crate::constants::NIL;
use crate::data::physics::raw_float::RawFloat;

/// The physics state an inventory item and a physic object carry in their update packets, `SPHNetState` behind the
/// byte `CSE_ALifeInventoryItem::UPDATE_Write` and `CSE_ALifeObjectPhysic::UPDATE_Write` open it with.
///
/// That byte packs the item count into its low five bits and, above them, a mask: whether the body is enabled, and
/// whether each velocity is left out as zero. A zero byte is a state that is not there at all, so an object without one
/// is `None` beside its other update fields rather than a state with dead fields. The SDK writes some states without
/// initialising them, so each float keeps the bits it is stored in.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicsNetState {
  /// The synchronised items, `m_u8NumItems`, at most the 31 its five bits hold.
  pub item_count: u8,
  pub enabled: bool,
  pub force: [RawFloat; 3],
  pub torque: [RawFloat; 3],
  /// Where the body stands; an item's `UPDATE_Read` moves the object here (`o_Position`).
  pub position: [RawFloat; 3],
  /// The orientation quaternion as x, y, z and w.
  pub rotation: [RawFloat; 4],
  /// `None` where the mask leaves the angular velocity out as zero.
  pub angular_velocity: Option<[RawFloat; 3]>,
  /// `None` where the mask leaves the linear velocity out as zero.
  pub linear_velocity: Option<[RawFloat; 3]>,
  /// The byte closing the state: zero freezes the object and anything else leaves it awake.
  pub awake: u8,
}

impl PhysicsNetState {
  /// The largest count the byte's five bits hold.
  pub const MAX_ITEM_COUNT: u8 = (1 << 5) - 1;

  /// What the engine writes to close a state, and what it takes a state ending its packet early to mean.
  pub const AWAKE: u8 = 1;

  /// `inventory_item_state_enabled`, above the five count bits.
  const FLAG_ENABLED: u8 = 1 << 5;
  /// `inventory_item_angular_null`.
  const FLAG_ANGULAR_NULL: u8 = 1 << 6;
  /// `inventory_item_linear_null`.
  const FLAG_LINEAR_NULL: u8 = 1 << 7;

  /// Where the state places the object.
  pub fn get_position(&self) -> Vector3d {
    Vector3d::new(self.position[0].get(), self.position[1].get(), self.position[2].get())
  }

  /// Place the object at `position`.
  pub fn set_position(&mut self, position: &Vector3d) {
    self.position = [position.x, position.y, position.z].map(RawFloat::from);
  }

  /// Read the state an update packet carries, or `None` for the zero byte standing in for one.
  ///
  /// A state ending its packet before the closing byte reads as awake, as the engine reads it.
  pub fn read_optional<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Option<Self>> {
    let packed: u8 = reader.read_u8()?;

    if packed == 0 {
      return Ok(None);
    }

    Ok(Some(Self {
      item_count: packed & Self::MAX_ITEM_COUNT,
      enabled: packed & Self::FLAG_ENABLED != 0,
      force: RawFloat::read_array::<T, D, 3>(reader)?,
      torque: RawFloat::read_array::<T, D, 3>(reader)?,
      position: RawFloat::read_array::<T, D, 3>(reader)?,
      rotation: RawFloat::read_array::<T, D, 4>(reader)?,
      angular_velocity: if packed & Self::FLAG_ANGULAR_NULL == 0 {
        Some(RawFloat::read_array::<T, D, 3>(reader)?)
      } else {
        None
      },
      linear_velocity: if packed & Self::FLAG_LINEAR_NULL == 0 {
        Some(RawFloat::read_array::<T, D, 3>(reader)?)
      } else {
        None
      },
      awake: if reader.is_ended() {
        Self::AWAKE
      } else {
        reader.read_u8()?
      },
    }))
  }

  /// Write the state, or the zero byte standing in for none.
  ///
  /// # Errors
  ///
  /// Refuses a count past [`Self::MAX_ITEM_COUNT`], and a state that would pack into the zero byte, which reads back
  /// as no state at all.
  pub fn write_optional<T: ByteOrder>(state: Option<&Self>, writer: &mut ChunkWriter) -> XrfResult {
    let Some(state) = state else {
      writer.write_u8(0)?;

      return Ok(());
    };

    writer.write_u8(state.get_packed()?)?;

    RawFloat::write_array::<T>(writer, &state.force)?;
    RawFloat::write_array::<T>(writer, &state.torque)?;
    RawFloat::write_array::<T>(writer, &state.position)?;
    RawFloat::write_array::<T>(writer, &state.rotation)?;

    if let Some(velocity) = &state.angular_velocity {
      RawFloat::write_array::<T>(writer, velocity)?;
    }

    if let Some(velocity) = &state.linear_velocity {
      RawFloat::write_array::<T>(writer, velocity)?;
    }

    writer.write_u8(state.awake)?;

    Ok(())
  }

  /// The byte opening the state: its item count, and the mask of what follows. A byte packing to zero would read
  /// back as no state at all.
  fn get_packed(&self) -> XrfResult<u8> {
    if self.item_count > Self::MAX_ITEM_COUNT {
      return Err(XrfError::new_invalid_error(format!(
        "Physics state item count {} does not fit its five bits",
        self.item_count
      )));
    }

    let mut packed: u8 = self.item_count;

    if self.enabled {
      packed |= Self::FLAG_ENABLED;
    }

    if self.angular_velocity.is_none() {
      packed |= Self::FLAG_ANGULAR_NULL;
    }

    if self.linear_velocity.is_none() {
      packed |= Self::FLAG_LINEAR_NULL;
    }

    if packed == 0 {
      return Err(XrfError::new_invalid_error(
        "Physics state with no items, disabled and with both velocities packs into the byte of an absent state",
      ));
    }

    Ok(packed)
  }

  /// Import the state kept under `prefix`, where `prefix = nil` stands for none.
  pub fn import_optional(section: &Section, prefix: &str) -> XrfResult<Option<Self>> {
    if section.get(prefix).map(str::trim) == Some(NIL) {
      return Ok(None);
    }

    Ok(Some(Self {
      item_count: section.read_field(&format!("{prefix}.item_count"))?,
      enabled: section.read_field(&format!("{prefix}.enabled"))?,
      force: RawFloat::import_array(section, &format!("{prefix}.force"))?,
      torque: RawFloat::import_array(section, &format!("{prefix}.torque"))?,
      position: RawFloat::import_array(section, &format!("{prefix}.position"))?,
      rotation: RawFloat::import_array(section, &format!("{prefix}.rotation"))?,
      angular_velocity: Self::import_velocity(section, &format!("{prefix}.angular_velocity"))?,
      linear_velocity: Self::import_velocity(section, &format!("{prefix}.linear_velocity"))?,
      awake: section.read_field(&format!("{prefix}.awake"))?,
    }))
  }

  /// Export the state under `prefix`, or `prefix = nil` for none.
  pub fn export_optional(state: Option<&Self>, section_name: &str, prefix: &str, ltx: &mut Ltx) {
    let Some(state) = state else {
      ltx.with_section(section_name).set(prefix, NIL);

      return;
    };

    ltx
      .with_section(section_name)
      .set(format!("{prefix}.item_count"), state.item_count.to_string())
      .set(format!("{prefix}.enabled"), state.enabled.to_string())
      .set(format!("{prefix}.force"), vector_to_string(&state.force))
      .set(format!("{prefix}.torque"), vector_to_string(&state.torque))
      .set(format!("{prefix}.position"), vector_to_string(&state.position))
      .set(format!("{prefix}.rotation"), vector_to_string(&state.rotation))
      .set(
        format!("{prefix}.angular_velocity"),
        Self::export_velocity(state.angular_velocity.as_ref()),
      )
      .set(
        format!("{prefix}.linear_velocity"),
        Self::export_velocity(state.linear_velocity.as_ref()),
      )
      .set(format!("{prefix}.awake"), state.awake.to_string());
  }

  fn import_velocity(section: &Section, key: &str) -> XrfResult<Option<[RawFloat; 3]>> {
    if section.get(key).map(str::trim) == Some(NIL) {
      return Ok(None);
    }

    Ok(Some(RawFloat::import_array(section, key)?))
  }

  fn export_velocity(velocity: Option<&[RawFloat; 3]>) -> String {
    velocity.map_or_else(|| String::from(NIL), |velocity| vector_to_string(velocity))
  }
}

#[cfg(test)]
impl PhysicsNetState {
  /// A state at `position` with both velocities left out, as most placed items carry one.
  pub fn new_mock(position: Vector3d) -> Self {
    let zero: [RawFloat; 3] = [RawFloat::from(0.0); 3];
    let mut state: Self = Self {
      item_count: 10,
      enabled: true,
      force: zero,
      torque: zero,
      position: zero,
      rotation: [0.0, 0.0, 0.0, 1.0].map(RawFloat::from),
      angular_velocity: None,
      linear_velocity: None,
      awake: Self::AWAKE,
    };

    state.set_position(&position);

    state
  }
}

#[cfg(test)]
mod tests {
  use byteorder::WriteBytesExt;
  use xrf_chunk::{ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;
  use xrf_math::Vector3d;

  use crate::data::physics::physics_net_state::PhysicsNetState;
  use crate::data::physics::raw_float::RawFloat;

  fn read_back(state: Option<&PhysicsNetState>) -> XrfResult<(usize, Option<PhysicsNetState>)> {
    let mut writer: ChunkWriter = ChunkWriter::new();

    PhysicsNetState::write_optional::<XRayByteOrder>(state, &mut writer)?;

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&writer.buffer)?;
    let read: Option<PhysicsNetState> = PhysicsNetState::read_optional::<XRayByteOrder, _>(&mut reader)?;

    reader.assert_read("Expect the whole state to be read")?;

    Ok((writer.buffer.len(), read))
  }

  /// The byte, three vectors, a quaternion and the closing byte: the 54-byte update most placed items carry.
  #[test]
  fn reads_and_writes_a_state_without_velocities() -> XrfResult {
    let state: PhysicsNetState = PhysicsNetState::new_mock(Vector3d::new(1.0, 2.0, 3.0));

    assert_eq!(read_back(Some(&state))?, (1 + 36 + 16 + 1, Some(state)));

    Ok(())
  }

  /// The mask decides which velocities follow, so each one present adds its three floats.
  #[test]
  fn reads_and_writes_the_velocities_the_mask_keeps() -> XrfResult {
    let mut state: PhysicsNetState = PhysicsNetState::new_mock(Vector3d::new(1.0, 2.0, 3.0));

    state.angular_velocity = Some([0.5, 0.0, 0.0].map(RawFloat::from));

    assert_eq!(read_back(Some(&state))?, (66, Some(state.clone())));

    state.enabled = false;
    state.linear_velocity = Some([0.0, -1.0, 0.0].map(RawFloat::from));

    assert_eq!(read_back(Some(&state))?, (78, Some(state)));

    Ok(())
  }

  /// A vanilla spawn holds states the SDK never initialised, every float of them `0xffffffff`.
  #[test]
  fn keeps_the_bits_of_an_uninitialised_state() -> XrfResult {
    let mut state: PhysicsNetState = PhysicsNetState::new_mock(Vector3d::ZERO);
    let unset: [RawFloat; 3] = [RawFloat(u32::MAX); 3];

    state.force = [RawFloat(0xffcc_031b), RawFloat::from(76.70584), RawFloat(0x7fff_4111)];
    state.torque = unset;
    state.position = unset;
    state.rotation = [RawFloat(u32::MAX); 4];
    state.angular_velocity = Some(unset);

    let (_, read) = read_back(Some(&state))?;

    assert_eq!(read, Some(state.clone()));

    let mut ltx: Ltx = Ltx::new();

    PhysicsNetState::export_optional(Some(&state), "object", "physic.update_state", &mut ltx);

    assert_eq!(
      PhysicsNetState::import_optional(
        ltx.section("object").expect("the object section"),
        "physic.update_state"
      )?,
      Some(state.clone())
    );
    assert_eq!(
      serde_json::from_str::<PhysicsNetState>(&serde_json::to_string(&state)?)?,
      state
    );

    Ok(())
  }

  #[test]
  fn reads_and_writes_the_zero_byte_of_no_state() -> XrfResult {
    assert_eq!(read_back(None)?, (1, None));

    Ok(())
  }

  /// A count of zero beside mask bits is not the zero byte, so the state still follows it.
  #[test]
  fn reads_a_state_counting_no_items() -> XrfResult {
    let mut state: PhysicsNetState = PhysicsNetState::new_mock(Vector3d::ZERO);

    state.item_count = 0;

    assert_eq!(read_back(Some(&state))?.1, Some(state));

    Ok(())
  }

  #[test]
  fn refuses_a_state_packing_into_the_zero_byte() {
    let mut state: PhysicsNetState = PhysicsNetState::new_mock(Vector3d::ZERO);

    state.item_count = 0;
    state.enabled = false;
    state.angular_velocity = Some([RawFloat::from(0.0); 3]);
    state.linear_velocity = Some([RawFloat::from(0.0); 3]);

    let error: String = read_back(Some(&state))
      .expect_err("a state packing to zero")
      .to_string();

    assert!(error.contains("packs into the byte of an absent state"), "{error}");
  }

  #[test]
  fn refuses_a_count_past_five_bits() {
    let mut state: PhysicsNetState = PhysicsNetState::new_mock(Vector3d::ZERO);

    state.item_count = PhysicsNetState::MAX_ITEM_COUNT + 1;

    let error: String = read_back(Some(&state)).expect_err("a count past five bits").to_string();

    assert!(error.contains("item count 32 does not fit"), "{error}");
  }

  /// `UPDATE_Read` takes a state ending its packet before the closing byte as awake.
  #[test]
  fn reads_a_state_ending_its_packet_as_awake() -> XrfResult {
    let state: PhysicsNetState = PhysicsNetState::new_mock(Vector3d::new(4.0, 5.0, 6.0));
    let mut writer: ChunkWriter = ChunkWriter::new();

    PhysicsNetState::write_optional::<XRayByteOrder>(Some(&state), &mut writer)?;
    writer.buffer.pop();

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&writer.buffer)?;

    assert_eq!(
      PhysicsNetState::read_optional::<XRayByteOrder, _>(&mut reader)?,
      Some(state)
    );

    Ok(())
  }

  #[test]
  fn keeps_a_frozen_closing_byte() -> XrfResult {
    let mut state: PhysicsNetState = PhysicsNetState::new_mock(Vector3d::ZERO);
    let mut writer: ChunkWriter = ChunkWriter::new();

    state.awake = 0;
    PhysicsNetState::write_optional::<XRayByteOrder>(Some(&state), &mut writer)?;
    writer.write_u8(42)?;

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&writer.buffer)?;

    assert_eq!(
      PhysicsNetState::read_optional::<XRayByteOrder, _>(&mut reader)?,
      Some(state)
    );

    Ok(())
  }

  #[test]
  fn moves_the_position_it_places_the_object_at() {
    let mut state: PhysicsNetState = PhysicsNetState::new_mock(Vector3d::ZERO);

    state.set_position(&Vector3d::new(-1.5, 2.0, 30.25));

    assert_eq!(state.get_position(), Vector3d::new(-1.5, 2.0, 30.25));
  }

  #[test]
  fn imports_and_exports_a_state_and_its_absence() -> XrfResult {
    let mut state: PhysicsNetState = PhysicsNetState::new_mock(Vector3d::new(1.5, -2.25, 3.0));

    state.rotation = [0.1, 0.2, 0.3, 0.9].map(RawFloat::from);
    state.linear_velocity = Some([0.0, -9.8, 0.0].map(RawFloat::from));

    let mut ltx: Ltx = Ltx::new();

    PhysicsNetState::export_optional(Some(&state), "object", "item.update_state", &mut ltx);
    PhysicsNetState::export_optional(None, "other", "item.update_state", &mut ltx);

    let source: Ltx = Ltx::read_from_str(&String::from_utf8_lossy(&ltx.write_to_bytes()?))?;

    assert_eq!(
      PhysicsNetState::import_optional(
        source.section("object").expect("the object section"),
        "item.update_state"
      )?,
      Some(state)
    );
    assert_eq!(
      PhysicsNetState::import_optional(source.section("other").expect("the other section"), "item.update_state")?,
      None
    );

    Ok(())
  }
}
