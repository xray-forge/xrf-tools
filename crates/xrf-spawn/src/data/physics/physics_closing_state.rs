use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReader, ChunkFieldWriter};
use xrf_error::XrfResult;
use xrf_ltx::{LtxSectionRead, LtxSectionWrite, LtxWrite};

use crate::data::physics::physics_net_state::PhysicsNetState;

/// The physics state closing a physic object's update packet (`CSE_ALifeObjectPhysic::UPDATE_Read`), kept as stored.
/// The engine reads an empty packet and a lone zero byte alike as no state.
///
/// A zero byte with bytes behind it is a state packing to zero, which the SDK's 2009 `xrSE_Factory` writes for an
/// object of no items with both velocities set; the engine reads the zero byte and skips the rest.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum PhysicsClosingState {
  /// The packet ends before the state's byte.
  Absent,
  /// The zero byte of no state, as the engine writes it.
  Unset,
  /// A state, including one the SDK packed to zero.
  Set(PhysicsNetState),
}

impl PhysicsClosingState {
  /// Serialized form of an absent state, distinct from `nil`, which is a written zero byte.
  pub const ABSENT: &'static str = "absent";

  /// The state to change, where the packet carries one.
  pub fn get_state_mut(&mut self) -> Option<&mut PhysicsNetState> {
    match self {
      Self::Set(state) => Some(state),
      Self::Absent | Self::Unset => None,
    }
  }

  /// Read the state closing the update packet.
  ///
  /// # Errors
  ///
  /// Returns an error when the state is cut short.
  pub(crate) fn read<R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Self> {
    if reader.is_ended() {
      return Ok(Self::Absent);
    }

    let packed: u8 = reader.read_u8()?;

    if packed == 0 && reader.is_ended() {
      return Ok(Self::Unset);
    }

    PhysicsNetState::read_after(reader, packed).map(Self::Set)
  }

  /// Write the state back as it was read.
  ///
  /// # Errors
  ///
  /// Returns an error for a count past [`PhysicsNetState::MAX_ITEM_COUNT`].
  pub(crate) fn write<W: ChunkFieldWriter>(&self, writer: &mut W) -> XrfResult {
    match self {
      Self::Absent => Ok(()),
      Self::Unset => writer.write_u8(0),
      Self::Set(state) => state.write_after(writer, state.pack()?),
    }
  }

  /// Import the state kept under `prefix`.
  ///
  /// # Errors
  ///
  /// Returns an error when a field of the state does not read.
  pub fn import(section: &impl LtxSectionRead, prefix: &str) -> XrfResult<Self> {
    if section.get(prefix).map(str::trim) == Some(Self::ABSENT) {
      return Ok(Self::Absent);
    }

    Ok(match PhysicsNetState::import_optional(section, prefix)? {
      Some(state) => Self::Set(state),
      None => Self::Unset,
    })
  }

  /// Export the state under `prefix`.
  pub fn export(&self, section_name: &str, prefix: &str, ltx: &mut impl LtxWrite) {
    match self {
      Self::Absent => {
        ltx.with_section(section_name).set(prefix, Self::ABSENT);
      }
      Self::Unset => PhysicsNetState::export_optional(None, section_name, prefix, ltx),
      Self::Set(state) => PhysicsNetState::export_optional(Some(state), section_name, prefix, ltx),
    }
  }
}
