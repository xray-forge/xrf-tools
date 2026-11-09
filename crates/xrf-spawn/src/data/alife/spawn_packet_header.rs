use byteorder::ByteOrder;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkFieldReadWrite, ChunkFieldReader, ChunkFieldWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxImportExport, LtxRead, LtxSectionRead, LtxSectionWrite, LtxWrite};
use xrf_math::Vector3d;
use xrf_utils::{decode_bytes_from_base64, encode_bytes_to_base64, narrow_format_size};

use crate::constants::NET_ACTION_SPAWN;

/// What every server entity's spawn packet opens with, `CSE_Abstract::Spawn_Write` (`xrServer_Object_Base.cpp:156`),
/// up to the size it patches in once its class's own fields, the state, are written.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpawnPacketHeader {
  pub id: u16,
  /// `M_SPAWN`, which the engine asserts on before reading anything else.
  pub net_action: u16,
  /// The config section the entity is built from.
  pub section: String,
  /// The name the level gives this one, `s_name_replace`.
  pub name: String,
  pub script_game_id: u8,
  pub script_rp: u8,
  pub position: Vector3d,
  pub direction: Vector3d,
  pub respawn_time: u16,
  pub parent_id: u16,
  pub phantom_id: u16,
  /// `s_flags`; the spawn version is stored only while `M_SPAWN_VERSION` is set.
  pub script_flags: u16,
  pub version: u16,
  pub game_type: u16,
  pub script_version: u16,
  /// The client object's saved state (`CSE_Abstract::client_data`), raw bytes carried as stored, which fields cannot
  /// hold.
  pub client_data: Vec<u8>,
  pub spawn_id: u16,
}

impl SpawnPacketHeader {
  /// `M_SPAWN_VERSION`: the packet stores its spawn version after the flags.
  pub const FLAG_SPAWN_VERSION: u16 = 1 << 5;

  /// The highest spawn version refused: `CSE_Abstract::Spawn_Read` reads the game type only above it, and the packets
  /// here are laid out as Call of Pripyat and the forks built on it write them, at 128.
  pub const MAX_UNSUPPORTED_VERSION: u16 = 120;

  /// Writes the class's own fields after the `u16` that `Spawn_Write` patches with their size, its own two bytes
  /// counted. The patch is a seek over the bytes alone (`NET_Packet::w_seek`), so fields keep the zero first written.
  ///
  /// # Errors
  ///
  /// Returns an error when `write` fails, or when the state outgrows its `u16` size.
  pub fn write_state<T: ByteOrder, W: ChunkFieldWriter>(
    writer: &mut W,
    write: impl FnOnce(&mut W) -> XrfResult,
  ) -> XrfResult {
    let opened: usize = writer.get_position();

    writer.write_u16::<T>(0)?;
    write(writer)?;
    writer.patch_u16::<T>(
      opened,
      narrow_format_size(writer.get_position() - opened, "spawn packet state")?,
    )
  }

  /// Reads the class's own fields after their size. Bytes must hold exactly the size they count; fields keep the zero
  /// written before the patch, which `Spawn_Read` passes on and no class reads.
  ///
  /// # Errors
  ///
  /// Returns an error when `read` fails, or, in bytes, when the size is less than its own two bytes or counts other
  /// than the fields `read` took.
  pub fn read_state<T: ByteOrder, R: ChunkFieldReader, V>(
    &self,
    reader: &mut R,
    read: impl FnOnce(&mut R) -> XrfResult<V>,
  ) -> XrfResult<V> {
    let opened: usize = reader.get_position();
    let size: u16 = reader.read_u16::<T>()?;

    if reader.is_field_form() {
      return read(reader);
    }

    if usize::from(size) < size_of::<u16>() {
      return Err(XrfError::new_invalid_error(format!(
        "Spawn packet '{}' state size {size} is less than its own two-byte prefix",
        self.name
      )));
    }

    let state: V = read(reader)?;
    let counted: usize = reader.get_position() - opened;

    if usize::from(size) != counted {
      return Err(XrfError::new_invalid_error(format!(
        "Spawn packet '{}' state counts {size} bytes with its size, its fields hold {counted}",
        self.name
      )));
    }

    Ok(state)
  }
}

/// As `CSE_Abstract::Spawn_Read` reads it, refusing another action than `M_SPAWN` and a version older than Call of
/// Pripyat's layout.
impl ChunkFieldReadWrite for SpawnPacketHeader {
  fn read_fields<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Self> {
    let net_action: u16 = reader.read_u16::<T>()?;

    if net_action != NET_ACTION_SPAWN {
      return Err(XrfError::new_parsing_error(format!(
        "Unexpected net action {net_action} in a spawn packet, expected {NET_ACTION_SPAWN}"
      )));
    }

    let section: String = reader.read_w1251_string()?;
    let name: String = reader.read_w1251_string()?;
    let script_game_id: u8 = reader.read_u8()?;
    let script_rp: u8 = reader.read_u8()?;
    let position: Vector3d = reader.read_xr::<T, _>()?;
    let direction: Vector3d = reader.read_xr::<T, _>()?;
    let respawn_time: u16 = reader.read_u16::<T>()?;
    let id: u16 = reader.read_u16::<T>()?;
    let parent_id: u16 = reader.read_u16::<T>()?;
    let phantom_id: u16 = reader.read_u16::<T>()?;
    let script_flags: u16 = reader.read_u16::<T>()?;
    let version: u16 = if script_flags & Self::FLAG_SPAWN_VERSION == 0 {
      0
    } else {
      reader.read_u16::<T>()?
    };

    if version <= Self::MAX_UNSUPPORTED_VERSION {
      return Err(XrfError::new_assertion_error(format!(
        "Unexpected version {version} of spawn packet '{name}', script flags {script_flags}"
      )));
    }

    let game_type: u16 = reader.read_u16::<T>()?;
    let script_version: u16 = reader.read_u16::<T>()?;
    let client_data_size: u16 = reader.read_u16::<T>()?;
    let client_data: Vec<u8> = reader.read_bytes(usize::from(client_data_size))?;

    Ok(Self {
      id,
      net_action,
      section,
      name,
      script_game_id,
      script_rp,
      position,
      direction,
      respawn_time,
      parent_id,
      phantom_id,
      script_flags,
      version,
      game_type,
      script_version,
      client_data,
      spawn_id: reader.read_u16::<T>()?,
    })
  }

  fn write_fields<T: ByteOrder, W: ChunkFieldWriter>(&self, writer: &mut W) -> XrfResult {
    writer.write_u16::<T>(self.net_action)?;
    writer.write_w1251_string(&self.section)?;
    writer.write_w1251_string(&self.name)?;
    writer.write_u8(self.script_game_id)?;
    writer.write_u8(self.script_rp)?;
    writer.write_xr::<T, _>(&self.position)?;
    writer.write_xr::<T, _>(&self.direction)?;
    writer.write_u16::<T>(self.respawn_time)?;
    writer.write_u16::<T>(self.id)?;
    writer.write_u16::<T>(self.parent_id)?;
    writer.write_u16::<T>(self.phantom_id)?;
    writer.write_u16::<T>(self.script_flags)?;

    if self.script_flags & Self::FLAG_SPAWN_VERSION != 0 {
      writer.write_u16::<T>(self.version)?;
    }

    writer.write_u16::<T>(self.game_type)?;
    writer.write_u16::<T>(self.script_version)?;
    writer.write_u16::<T>(narrow_format_size(self.client_data.len(), "spawn packet client data")?)?;
    writer.write_bytes(&self.client_data)?;
    writer.write_u16::<T>(self.spawn_id)
  }
}

impl LtxImportExport for SpawnPacketHeader {
  fn import<L: LtxRead>(section_name: &str, ltx: &L) -> XrfResult<Self> {
    let section: &L::Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "Spawn packet '{section_name}' should be defined in ltx file ({})",
        file!()
      ))
    })?;

    Ok(Self {
      id: section.read_field("id")?,
      net_action: section.read_field("net_action")?,
      section: section.read_field("section")?,
      name: section.read_field("name")?,
      script_game_id: section.read_field("script_game_id")?,
      script_rp: section.read_field("script_rp")?,
      position: section.read_field("position")?,
      direction: section.read_field("direction")?,
      respawn_time: section.read_field("respawn_time")?,
      parent_id: section.read_field("parent_id")?,
      phantom_id: section.read_field("phantom_id")?,
      script_flags: section.read_field("script_flags")?,
      version: section.read_field("version")?,
      game_type: section.read_field("game_type")?,
      script_version: section.read_field("script_version")?,
      client_data: decode_bytes_from_base64(&section.read_field::<String>("client_data")?)?,
      spawn_id: section.read_field("spawn_id")?,
    })
  }

  fn export(&self, section_name: &str, ltx: &mut impl LtxWrite) -> XrfResult {
    ltx
      .with_section(section_name)
      .set("id", self.id)
      .set("net_action", self.net_action)
      .set("section", &self.section)
      .set("name", &self.name)
      .set("script_game_id", self.script_game_id)
      .set("script_rp", self.script_rp)
      .set("position", self.position)
      .set("direction", self.direction)
      .set("respawn_time", self.respawn_time)
      .set("parent_id", self.parent_id)
      .set("phantom_id", self.phantom_id)
      .set("script_flags", self.script_flags)
      .set("version", self.version)
      .set("game_type", self.game_type)
      .set("script_version", self.script_version)
      .set("client_data", encode_bytes_to_base64(&self.client_data))
      .set("spawn_id", self.spawn_id);

    Ok(())
  }
}
