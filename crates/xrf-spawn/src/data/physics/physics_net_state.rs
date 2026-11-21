use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReadWriteOptional, ChunkFieldReader, ChunkFieldWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxSectionRead, LtxSectionWrite, LtxWrite};
use xrf_math::Vector3d;
use xrf_utils::vector_to_string;

use crate::constants::NIL;
use crate::data::physics::raw_float::RawFloat;

/// The physics state, `SPHNetState`, an inventory item's and a physic object's update packets carry behind an opening
/// byte (`CSE_ALifeInventoryItem::UPDATE_Write`, `CSE_ALifeObjectPhysic::UPDATE_Write`).
///
/// That byte packs the item count into its low five bits and a mask above them: whether the body is enabled, and
/// whether each velocity is left out as zero. A zero byte means no state, so an object without one holds `None`; a
/// physic object's closing state differs (`PhysicsClosingState`). The SDK writes some states uninitialised, so each
/// float keeps its stored bits.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
  /// The closing byte: zero freezes the object, anything else leaves it awake. `None` where the packet ends before it,
  /// which the engine reads as awake (`r_eof`) and which writes back without it.
  pub awake: Option<u8>,
}

impl PhysicsNetState {
  /// The largest count the byte's five bits hold.
  pub const MAX_ITEM_COUNT: u8 = (1 << 5) - 1;

  /// What the engine writes to close a state.
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

  /// The opening byte, refused where it packs to zero, which reads back as no state.
  fn get_packed(&self) -> XrfResult<u8> {
    let packed: u8 = self.pack()?;

    if packed == 0 {
      return Err(XrfError::new_invalid_error(
        "Physics state with no items, disabled and with both velocities packs into the byte of an absent state",
      ));
    }

    Ok(packed)
  }

  /// The count and mask packed into the opening byte, whatever they come to.
  pub(crate) fn pack(&self) -> XrfResult<u8> {
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

    Ok(packed)
  }

  /// The state behind its opening byte `packed`; its closing byte is `None` where the packet ends first.
  pub(crate) fn read_after<R: ChunkFieldReader>(reader: &mut R, packed: u8) -> XrfResult<Self> {
    Ok(Self {
      item_count: packed & Self::MAX_ITEM_COUNT,
      enabled: packed & Self::FLAG_ENABLED != 0,
      force: RawFloat::read_array::<_, 3>(reader)?,
      torque: RawFloat::read_array::<_, 3>(reader)?,
      position: RawFloat::read_array::<_, 3>(reader)?,
      rotation: RawFloat::read_array::<_, 4>(reader)?,
      angular_velocity: if packed & Self::FLAG_ANGULAR_NULL == 0 {
        Some(RawFloat::read_array::<_, 3>(reader)?)
      } else {
        None
      },
      linear_velocity: if packed & Self::FLAG_LINEAR_NULL == 0 {
        Some(RawFloat::read_array::<_, 3>(reader)?)
      } else {
        None
      },
      awake: if reader.is_ended() {
        None
      } else {
        Some(reader.read_u8()?)
      },
    })
  }

  /// Writes the opening byte `packed` and the state behind it, as read.
  pub(crate) fn write_after<W: ChunkFieldWriter>(&self, writer: &mut W, packed: u8) -> XrfResult {
    writer.write_u8(packed)?;

    RawFloat::write_array(writer, &self.force)?;
    RawFloat::write_array(writer, &self.torque)?;
    RawFloat::write_array(writer, &self.position)?;
    RawFloat::write_array(writer, &self.rotation)?;

    if let Some(velocity) = &self.angular_velocity {
      RawFloat::write_array(writer, velocity)?;
    }

    if let Some(velocity) = &self.linear_velocity {
      RawFloat::write_array(writer, velocity)?;
    }

    match self.awake {
      Some(awake) => writer.write_u8(awake),
      None => Ok(()),
    }
  }
}

/// The state an update packet carries, or the zero byte standing in for none.
impl ChunkFieldReadWriteOptional for PhysicsNetState {
  /// A state ending its packet before the closing byte reads without it.
  fn read_field_optional<R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Option<Self>> {
    let packed: u8 = reader.read_u8()?;

    if packed == 0 {
      return Ok(None);
    }

    Self::read_after(reader, packed).map(Some)
  }

  /// Refuses a count past [`Self::MAX_ITEM_COUNT`], and a state packing into the zero byte of no state.
  fn write_field_optional<W: ChunkFieldWriter>(writer: &mut W, state: Option<&Self>) -> XrfResult {
    match state {
      Some(state) => state.write_after(writer, state.get_packed()?),
      None => writer.write_u8(0),
    }
  }
}

impl PhysicsNetState {
  /// Import the state kept under `prefix`, where `prefix = nil` stands for none.
  pub fn import_optional(section: &impl LtxSectionRead, prefix: &str) -> XrfResult<Option<Self>> {
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
      awake: Self::import_awake(section, &format!("{prefix}.awake"))?,
    }))
  }

  /// Export the state under `prefix`, or `prefix = nil` for none.
  pub fn export_optional(state: Option<&Self>, section_name: &str, prefix: &str, ltx: &mut impl LtxWrite) {
    let Some(state) = state else {
      ltx.with_section(section_name).set(prefix, NIL);

      return;
    };

    ltx
      .with_section(section_name)
      .set(format!("{prefix}.item_count"), state.item_count)
      .set(format!("{prefix}.enabled"), state.enabled)
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
      .set(
        format!("{prefix}.awake"),
        state.awake.map_or_else(|| String::from(NIL), |awake| awake.to_string()),
      );
  }

  /// The closing byte under `key`, where `nil`, or no key, stands for a packet ending before it.
  fn import_awake(section: &impl LtxSectionRead, key: &str) -> XrfResult<Option<u8>> {
    match section.get(key).map(str::trim) {
      None | Some(NIL) => Ok(None),
      Some(_) => section.read_field(key).map(Some),
    }
  }

  fn import_velocity(section: &impl LtxSectionRead, key: &str) -> XrfResult<Option<[RawFloat; 3]>> {
    if section.get(key).map(str::trim) == Some(NIL) {
      return Ok(None);
    }

    Ok(Some(RawFloat::import_array(section, key)?))
  }

  fn export_velocity(velocity: Option<&[RawFloat; 3]>) -> String {
    velocity.map_or_else(|| String::from(NIL), |velocity| vector_to_string(velocity))
  }
}

#[cfg(any(test, feature = "mocks"))]
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
      awake: Some(Self::AWAKE),
    };

    state.set_position(&position);

    state
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReadWriteOptional, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayWrite};
  use xrf_error::{MapXrfErr, XrfResult};
  use xrf_ltx::Ltx;
  use xrf_math::Vector3d;

  use crate::data::physics::physics_net_state::PhysicsNetState;
  use crate::data::physics::raw_float::RawFloat;

  fn read_back(state: Option<&PhysicsNetState>) -> XrfResult<(usize, Option<PhysicsNetState>)> {
    let mut writer: ChunkWriter = ChunkWriter::new();

    PhysicsNetState::write_optional(&mut writer, state)?;

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&writer.buffer)?;
    let read: Option<PhysicsNetState> = PhysicsNetState::read_optional(&mut reader)?;

    reader.assert_read("Expect the whole state to be read")?;

    Ok((writer.buffer.len(), read))
  }

  // The 54-byte update most placed items carry.
  #[test]
  fn reads_and_writes_a_state_without_velocities() -> XrfResult {
    let state: PhysicsNetState = PhysicsNetState::new_mock(Vector3d::new(1.0, 2.0, 3.0));

    assert_eq!(read_back(Some(&state))?, (1 + 36 + 16 + 1, Some(state)));

    Ok(())
  }

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

  // Vanilla spawns hold states the SDK never initialised, their floats `0xffffffff`.
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
      serde_json::from_str::<PhysicsNetState>(&serde_json::to_string(&state).map_serde_err()?).map_serde_err()?,
      state
    );

    Ok(())
  }

  #[test]
  fn reads_and_writes_the_zero_byte_of_no_state() -> XrfResult {
    assert_eq!(read_back(None)?, (1, None));

    Ok(())
  }

  // The mask bits keep a zero count off the zero byte.
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

  // `UPDATE_Read` takes such a state as awake.
  #[test]
  fn keeps_a_state_ending_its_packet_before_the_closing_byte() -> XrfResult {
    let mut state: PhysicsNetState = PhysicsNetState::new_mock(Vector3d::new(4.0, 5.0, 6.0));

    state.awake = None;

    assert_eq!(read_back(Some(&state))?, (1 + 36 + 16, Some(state)));

    Ok(())
  }

  #[test]
  fn keeps_a_frozen_closing_byte() -> XrfResult {
    let mut state: PhysicsNetState = PhysicsNetState::new_mock(Vector3d::ZERO);
    let mut writer: ChunkWriter = ChunkWriter::new();

    state.awake = Some(0);
    PhysicsNetState::write_optional(&mut writer, Some(&state))?;
    writer.write_u8(42)?;

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&writer.buffer)?;

    assert_eq!(PhysicsNetState::read_optional(&mut reader)?, Some(state));

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

    let mut short: PhysicsNetState = state.clone();

    short.awake = None;

    let mut ltx: Ltx = Ltx::new();

    PhysicsNetState::export_optional(Some(&state), "object", "item.update_state", &mut ltx);
    PhysicsNetState::export_optional(Some(&short), "short", "item.update_state", &mut ltx);
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
      PhysicsNetState::import_optional(source.section("short").expect("the short section"), "item.update_state")?,
      Some(short)
    );
    assert_eq!(
      PhysicsNetState::import_optional(source.section("other").expect("the other section"), "item.update_state")?,
      None
    );

    Ok(())
  }
}
