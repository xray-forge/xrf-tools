use std::io::Write;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, Section};
use xrf_math::Vector3d;
use xrf_utils::{decode_bytes_from_base64, encode_bytes_to_base64, vector_to_string};

use crate::data::physics::raw_float::RawFloat;

/// The body an actor's update packet saves, behind the `u16` count `CSE_ALifeCreatureActor::UPDATE_Write` opens it with:
/// nothing at zero, the living body's physics state at one, and a dead body's bones above that.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all_fields = "camelCase")]
pub enum ActorPhysicsState {
  /// No body was saved.
  Absent,
  /// The living body, `m_AliveState`, in the order the actor writes it.
  Alive {
    /// The body's enabled flag, as the byte the engine reads it into.
    enabled: u8,
    angular_velocity: [RawFloat; 3],
    linear_velocity: [RawFloat; 3],
    force: [RawFloat; 3],
    torque: [RawFloat; 3],
    position: [RawFloat; 3],
    /// The orientation quaternion as x, y, z and w.
    rotation: [RawFloat; 4],
  },
  /// A dead body, `m_DeadBodyData` as `CActor::net_ExportDeadBody` writes it: the low and high corners its bones are
  /// quantised within, then `bone_data_size` bytes for each of its `bone_count` bones.
  Dead {
    bone_count: u16,
    bone_data_size: u8,
    low_corner: [RawFloat; 3],
    high_corner: [RawFloat; 3],
    bones: Vec<u8>,
  },
}

impl ActorPhysicsState {
  /// The bytes `m_DeadBodyData` holds, past which the engine's reader would overrun it.
  pub const MAX_DEAD_BODY_SIZE: usize = 1024;

  /// The two corners a dead body's bones are quantised within, ahead of the bones.
  const DEAD_BODY_CORNERS_SIZE: usize = 24;

  /// The ltx spelling of each kind of body.
  const ABSENT: &'static str = "absent";
  const ALIVE: &'static str = "alive";
  const DEAD: &'static str = "dead";

  /// Move the body by `offset`: the living body's position, or both corners a dead body's bones are quantised within,
  /// which carries every bone with them.
  pub fn translate(&mut self, offset: &Vector3d) {
    match self {
      Self::Absent => {}
      Self::Alive { position, .. } => RawFloat::translate(position, offset),
      Self::Dead {
        low_corner,
        high_corner,
        ..
      } => {
        RawFloat::translate(low_corner, offset);
        RawFloat::translate(high_corner, offset);
      }
    }
  }

  /// Read the body an actor's update packet saves.
  ///
  /// # Errors
  ///
  /// Refuses a dead body larger than the engine's buffer for one.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(match reader.read_u16::<T>()? {
      0 => Self::Absent,
      1 => Self::Alive {
        enabled: reader.read_u8()?,
        angular_velocity: RawFloat::read_array::<T, D, 3>(reader)?,
        linear_velocity: RawFloat::read_array::<T, D, 3>(reader)?,
        force: RawFloat::read_array::<T, D, 3>(reader)?,
        torque: RawFloat::read_array::<T, D, 3>(reader)?,
        position: RawFloat::read_array::<T, D, 3>(reader)?,
        rotation: RawFloat::read_array::<T, D, 4>(reader)?,
      },
      bone_count => {
        let bone_data_size: u8 = reader.read_u8()?;
        let bones_size: usize = Self::get_bones_size(bone_count, bone_data_size)?;

        Self::Dead {
          bone_count,
          bone_data_size,
          low_corner: RawFloat::read_array::<T, D, 3>(reader)?,
          high_corner: RawFloat::read_array::<T, D, 3>(reader)?,
          bones: reader.read_bytes(bones_size)?,
        }
      }
    })
  }

  /// Write the body as an actor's update packet saves it.
  ///
  /// # Errors
  ///
  /// Refuses a dead body of fewer than two bones, which reads back as another state, one whose bones are not the size
  /// its counts give, and one larger than the engine's buffer for it.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    match self {
      Self::Absent => writer.write_u16::<T>(0)?,
      Self::Alive {
        enabled,
        angular_velocity,
        linear_velocity,
        force,
        torque,
        position,
        rotation,
      } => {
        writer.write_u16::<T>(1)?;
        writer.write_u8(*enabled)?;

        for floats in [angular_velocity, linear_velocity, force, torque, position] {
          RawFloat::write_array::<T>(writer, floats)?;
        }

        RawFloat::write_array::<T>(writer, rotation)?;
      }
      Self::Dead {
        bone_count,
        bone_data_size,
        low_corner,
        high_corner,
        bones,
      } => {
        if *bone_count < 2 {
          return Err(XrfError::new_invalid_error(format!(
            "Actor dead body of {bone_count} bones writes the count of another state"
          )));
        }

        let bones_size: usize = Self::get_bones_size(*bone_count, *bone_data_size)?;

        if bones.len() != bones_size {
          return Err(XrfError::new_invalid_error(format!(
            "Actor dead body of {bone_count} bones of {bone_data_size} bytes holds {} bytes of bones rather than \
             {bones_size}",
            bones.len()
          )));
        }

        writer.write_u16::<T>(*bone_count)?;
        writer.write_u8(*bone_data_size)?;
        RawFloat::write_array::<T>(writer, low_corner)?;
        RawFloat::write_array::<T>(writer, high_corner)?;
        writer.write_all(bones)?;
      }
    }

    Ok(())
  }

  /// The bytes `bone_count` bones of `bone_data_size` each occupy, refusing a body that with its corners, `24 + size *
  /// count` as the engine counts it, would overrun the engine's buffer.
  fn get_bones_size(bone_count: u16, bone_data_size: u8) -> XrfResult<usize> {
    let bones_size: usize = usize::from(bone_data_size) * usize::from(bone_count);

    if Self::DEAD_BODY_CORNERS_SIZE + bones_size > Self::MAX_DEAD_BODY_SIZE {
      return Err(XrfError::new_invalid_error(format!(
        "Actor dead body of {bone_count} bones of {bone_data_size} bytes overruns the engine's {} byte buffer",
        Self::MAX_DEAD_BODY_SIZE
      )));
    }

    Ok(bones_size)
  }

  /// Import the body kept under `prefix`, its kind in `prefix` itself.
  pub fn import(section: &Section, prefix: &str) -> XrfResult<Self> {
    let key = |name: &str| format!("{prefix}.{name}");

    Ok(match section.read_field::<String>(prefix)?.trim() {
      Self::ABSENT => Self::Absent,
      Self::ALIVE => Self::Alive {
        enabled: section.read_field(&key("enabled"))?,
        angular_velocity: RawFloat::import_array(section, &key("angular_velocity"))?,
        linear_velocity: RawFloat::import_array(section, &key("linear_velocity"))?,
        force: RawFloat::import_array(section, &key("force"))?,
        torque: RawFloat::import_array(section, &key("torque"))?,
        position: RawFloat::import_array(section, &key("position"))?,
        rotation: RawFloat::import_array(section, &key("rotation"))?,
      },
      Self::DEAD => Self::Dead {
        bone_count: section.read_field(&key("bone_count"))?,
        bone_data_size: section.read_field(&key("bone_data_size"))?,
        low_corner: RawFloat::import_array(section, &key("low_corner"))?,
        high_corner: RawFloat::import_array(section, &key("high_corner"))?,
        bones: decode_bytes_from_base64(&section.read_field::<String>(&key("bones"))?)?,
      },
      kind => {
        return Err(XrfError::new_parsing_error(format!(
          "Unknown actor body '{kind}' in '{prefix}', expected {}, {} or {}",
          Self::ABSENT,
          Self::ALIVE,
          Self::DEAD
        )));
      }
    })
  }

  /// Export the body under `prefix`, its kind in `prefix` itself.
  pub fn export(&self, section_name: &str, prefix: &str, ltx: &mut Ltx) {
    let key = |name: &str| format!("{prefix}.{name}");
    let mut section = ltx.with_section(section_name);

    match self {
      Self::Absent => {
        section.set(prefix, Self::ABSENT);
      }
      Self::Alive {
        enabled,
        angular_velocity,
        linear_velocity,
        force,
        torque,
        position,
        rotation,
      } => {
        section
          .set(prefix, Self::ALIVE)
          .set(key("enabled"), enabled.to_string())
          .set(key("angular_velocity"), vector_to_string(angular_velocity))
          .set(key("linear_velocity"), vector_to_string(linear_velocity))
          .set(key("force"), vector_to_string(force))
          .set(key("torque"), vector_to_string(torque))
          .set(key("position"), vector_to_string(position))
          .set(key("rotation"), vector_to_string(rotation));
      }
      Self::Dead {
        bone_count,
        bone_data_size,
        low_corner,
        high_corner,
        bones,
      } => {
        section
          .set(prefix, Self::DEAD)
          .set(key("bone_count"), bone_count.to_string())
          .set(key("bone_data_size"), bone_data_size.to_string())
          .set(key("low_corner"), vector_to_string(low_corner))
          .set(key("high_corner"), vector_to_string(high_corner))
          .set(key("bones"), encode_bytes_to_base64(bones));
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_ltx::Ltx;
  use xrf_math::Vector3d;

  use crate::data::physics::actor_physics_state::ActorPhysicsState;
  use crate::data::physics::raw_float::RawFloat;

  fn read_back(state: &ActorPhysicsState) -> XrfResult<(usize, ActorPhysicsState)> {
    let mut writer: ChunkWriter = ChunkWriter::new();

    state.write::<XRayByteOrder>(&mut writer)?;

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&writer.buffer)?;
    let read: ActorPhysicsState = ActorPhysicsState::read::<XRayByteOrder, _>(&mut reader)?;

    reader.assert_read("Expect the whole body to be read")?;

    Ok((writer.buffer.len(), read))
  }

  /// Three bones of ten bytes, as `CActor::net_ExportDeadBody` writes each: position, rotation and a velocity point.
  fn new_dead_body() -> ActorPhysicsState {
    ActorPhysicsState::Dead {
      bone_count: 3,
      bone_data_size: 10,
      low_corner: [-1.0, 0.0, -1.0].map(RawFloat::from),
      high_corner: [1.0, 2.0, 1.0].map(RawFloat::from),
      bones: (0..30).collect(),
    }
  }

  #[test]
  fn reads_and_writes_each_kind_of_body() -> XrfResult {
    let alive: ActorPhysicsState = ActorPhysicsState::Alive {
      enabled: 1,
      angular_velocity: [RawFloat::from(0.0); 3],
      linear_velocity: [RawFloat::from(0.0); 3],
      force: [RawFloat(u32::MAX); 3],
      torque: [RawFloat(0xffcc_031b); 3],
      position: [1.0, 2.0, 3.0].map(RawFloat::from),
      rotation: [0.0, 0.0, 0.0, 1.0].map(RawFloat::from),
    };

    assert_eq!(read_back(&ActorPhysicsState::Absent)?, (2, ActorPhysicsState::Absent));
    assert_eq!(read_back(&alive)?, (2 + 1 + 5 * 12 + 16, alive));
    assert_eq!(read_back(&new_dead_body())?, (2 + 1 + 24 + 30, new_dead_body()));

    Ok(())
  }

  /// Moving both corners moves every bone quantised within them.
  #[test]
  fn translates_a_dead_body_by_its_corners() {
    let mut body: ActorPhysicsState = new_dead_body();

    body.translate(&Vector3d::new(10.0, -1.0, 0.5));

    assert_eq!(
      body,
      ActorPhysicsState::Dead {
        bone_count: 3,
        bone_data_size: 10,
        low_corner: [9.0, -1.0, -0.5].map(RawFloat::from),
        high_corner: [11.0, 1.0, 1.5].map(RawFloat::from),
        bones: (0..30).collect(),
      }
    );
  }

  #[test]
  fn refuses_a_dead_body_past_the_engine_buffer() {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let body: ActorPhysicsState = ActorPhysicsState::Dead {
      bone_count: 101,
      bone_data_size: 10,
      low_corner: [RawFloat::from(0.0); 3],
      high_corner: [RawFloat::from(0.0); 3],
      bones: vec![0; 1010],
    };

    let error: String = body
      .write::<XRayByteOrder>(&mut writer)
      .expect_err("1034 bytes")
      .to_string();

    assert!(error.contains("overruns the engine's 1024 byte buffer"), "{error}");
  }

  #[test]
  fn refuses_a_dead_body_another_count_would_read() {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let body: ActorPhysicsState = ActorPhysicsState::Dead {
      bone_count: 1,
      bone_data_size: 10,
      low_corner: [RawFloat::from(0.0); 3],
      high_corner: [RawFloat::from(0.0); 3],
      bones: vec![0; 10],
    };

    let error: String = body
      .write::<XRayByteOrder>(&mut writer)
      .expect_err("one bone")
      .to_string();

    assert!(error.contains("writes the count of another state"), "{error}");
  }

  #[test]
  fn imports_and_exports_each_kind_of_body() -> XrfResult {
    let mut ltx: Ltx = Ltx::new();

    ActorPhysicsState::Absent.export("absent", "actor.update_body", &mut ltx);
    new_dead_body().export("dead", "actor.update_body", &mut ltx);

    assert_eq!(
      ActorPhysicsState::import(ltx.section("absent").expect("a section"), "actor.update_body")?,
      ActorPhysicsState::Absent
    );
    assert_eq!(
      ActorPhysicsState::import(ltx.section("dead").expect("a section"), "actor.update_body")?,
      new_dead_body()
    );

    Ok(())
  }
}
