use std::ops::Deref;
use std::str::FromStr;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReadWriteList, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{Ltx, Section};
use xrf_utils::{assert_equal, assert_length};

use crate::constants::META_TYPE_FIELD;
use crate::data::particles::actions::particle_action_avoid::ParticleActionAvoid;
use crate::data::particles::actions::particle_action_bounce::ParticleActionBounce;
use crate::data::particles::actions::particle_action_copy_vertex::ParticleActionCopyVertex;
use crate::data::particles::actions::particle_action_damping::ParticleActionDamping;
use crate::data::particles::actions::particle_action_explosion::ParticleActionExplosion;
use crate::data::particles::actions::particle_action_follow::ParticleActionFollow;
use crate::data::particles::actions::particle_action_gravitate::ParticleActionGravitate;
use crate::data::particles::actions::particle_action_gravity::ParticleActionGravity;
use crate::data::particles::actions::particle_action_jet::ParticleActionJet;
use crate::data::particles::actions::particle_action_kill_old::ParticleActionKillOld;
use crate::data::particles::actions::particle_action_match_velocity::ParticleActionMatchVelocity;
use crate::data::particles::actions::particle_action_move::ParticleActionMove;
use crate::data::particles::actions::particle_action_orbit_line::ParticleActionOrbitLine;
use crate::data::particles::actions::particle_action_orbit_point::ParticleActionOrbitPoint;
use crate::data::particles::actions::particle_action_random_acceleration::ParticleActionRandomAcceleration;
use crate::data::particles::actions::particle_action_random_displace::ParticleActionRandomDisplace;
use crate::data::particles::actions::particle_action_random_velocity::ParticleActionRandomVelocity;
use crate::data::particles::actions::particle_action_restore::ParticleActionRestore;
use crate::data::particles::actions::particle_action_scatter::ParticleActionScatter;
use crate::data::particles::actions::particle_action_sink::ParticleActionSink;
use crate::data::particles::actions::particle_action_sink_velocity::ParticleActionSinkVelocity;
use crate::data::particles::actions::particle_action_source::ParticleActionSource;
use crate::data::particles::actions::particle_action_speed_limit::ParticleActionSpeedLimit;
use crate::data::particles::actions::particle_action_target_color::ParticleActionTargetColor;
use crate::data::particles::actions::particle_action_target_rotate::ParticleActionTargetRotate;
use crate::data::particles::actions::particle_action_target_size::ParticleActionTargetSize;
use crate::data::particles::actions::particle_action_target_velocity::ParticleActionTargetVelocity;
use crate::data::particles::actions::particle_action_turbulence::ParticleActionTurbulence;
use crate::data::particles::actions::particle_action_vortex::ParticleActionVortex;
use crate::data::particles::particle_action_type::ParticleActionType;
use crate::export::LtxImportExport;
use crate::file_import::read_ltx_field;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ParticleAction {
  Avoid(Box<ParticleActionAvoid>),
  Bounce(Box<ParticleActionBounce>),
  CopyVertex(Box<ParticleActionCopyVertex>),
  Damping(Box<ParticleActionDamping>),
  Explosion(Box<ParticleActionExplosion>),
  Follow(Box<ParticleActionFollow>),
  Gravitate(Box<ParticleActionGravitate>),
  Gravity(Box<ParticleActionGravity>),
  Jet(Box<ParticleActionJet>),
  KillOld(Box<ParticleActionKillOld>),
  MatchVelocity(Box<ParticleActionMatchVelocity>),
  Move(Box<ParticleActionMove>),
  OrbitLine(Box<ParticleActionOrbitLine>),
  OrbitPoint(Box<ParticleActionOrbitPoint>),
  RandomAccel(Box<ParticleActionRandomAcceleration>),
  RandomDisplace(Box<ParticleActionRandomDisplace>),
  RandomVelocity(Box<ParticleActionRandomVelocity>),
  Restore(Box<ParticleActionRestore>),
  Sink(Box<ParticleActionSink>),
  SinkVelocity(Box<ParticleActionSinkVelocity>),
  Source(Box<ParticleActionSource>),
  SpeedLimit(Box<ParticleActionSpeedLimit>),
  TargetColor(Box<ParticleActionTargetColor>),
  TargetSize(Box<ParticleActionTargetSize>),
  TargetRotate(Box<ParticleActionTargetRotate>),
  TargetVelocity(Box<ParticleActionTargetVelocity>),
  Vortex(Box<ParticleActionVortex>),
  Turbulence(Box<ParticleActionTurbulence>),
  Scatter(Box<ParticleActionScatter>),
}

impl ParticleAction {
  pub const META_TYPE: &'static str = "particle_action";

  /// The action type selects the payload that follows it, so only the type itself is unconditional.
  pub const MIN_SERIALIZED_SIZE: u64 = 4;
}

impl ChunkReadWriteList for ParticleAction {
  /// Read list of particle action data from chunk reader.
  fn read_list<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Vec<Self>> {
    let count: u32 = reader.read_u32::<T>()?;

    let mut actions: Vec<Self> = reader.new_bounded_vec(count.into(), Self::MIN_SERIALIZED_SIZE, "particle actions")?;

    for _ in 0..count {
      actions.push(
        reader
          .read_xr::<T, _>()
          .map_err(|error| XrfError::new_parsing_error(format!("Failed to read particle effect action: {}", error)))?,
      );
    }

    assert_length(
      &actions,
      count as usize,
      "Should read same count of action as declared in chunk",
    )?;
    reader.assert_read("Expect particle actions list chunk to be ended")?;

    Ok(actions)
  }

  /// Write particle action data into chunk writer.
  fn write_list<T: ByteOrder>(writer: &mut ChunkWriter, actions: &[Self]) -> XrfResult {
    writer.write_u32::<T>(actions.len() as u32)?;

    for action in actions {
      writer.write_xr::<T, _>(action)?;
    }

    Ok(())
  }
}

impl ChunkReadWrite for ParticleAction {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let action_type: ParticleActionType = ParticleActionType::from(reader.read_u32::<T>()?);

    Ok(match action_type {
      ParticleActionType::Avoid => Self::Avoid(Box::new(reader.read_xr::<T, _>()?)),
      ParticleActionType::Bounce => Self::Bounce(Box::new(reader.read_xr::<T, _>()?)),
      ParticleActionType::CopyVertex => Self::CopyVertex(Box::new(reader.read_xr::<T, _>()?)),
      ParticleActionType::Damping => Self::Damping(Box::new(reader.read_xr::<T, _>()?)),
      ParticleActionType::Explosion => Self::Explosion(Box::new(reader.read_xr::<T, _>()?)),
      ParticleActionType::Follow => Self::Follow(Box::new(reader.read_xr::<T, _>()?)),
      ParticleActionType::Gravitate => Self::Gravitate(Box::new(reader.read_xr::<T, _>()?)),
      ParticleActionType::Gravity => Self::Gravity(Box::new(reader.read_xr::<T, _>()?)),
      ParticleActionType::Jet => Self::Jet(Box::new(reader.read_xr::<T, _>()?)),
      ParticleActionType::KillOld => Self::KillOld(Box::new(reader.read_xr::<T, _>()?)),
      ParticleActionType::MatchVelocity => Self::MatchVelocity(Box::new(reader.read_xr::<T, _>()?)),
      ParticleActionType::Move => Self::Move(Box::new(reader.read_xr::<T, _>()?)),
      ParticleActionType::OrbitLine => Self::OrbitLine(Box::new(reader.read_xr::<T, _>()?)),
      ParticleActionType::OrbitPoint => Self::OrbitPoint(Box::new(reader.read_xr::<T, _>()?)),
      ParticleActionType::RandomAccel => Self::RandomAccel(Box::new(reader.read_xr::<T, _>()?)),
      ParticleActionType::RandomDisplace => Self::RandomDisplace(Box::new(reader.read_xr::<T, _>()?)),
      ParticleActionType::RandomVelocity => Self::RandomVelocity(Box::new(reader.read_xr::<T, _>()?)),
      ParticleActionType::Restore => Self::Restore(Box::new(reader.read_xr::<T, _>()?)),
      ParticleActionType::Sink => Self::Sink(Box::new(reader.read_xr::<T, _>()?)),
      ParticleActionType::SinkVelocity => Self::SinkVelocity(Box::new(reader.read_xr::<T, _>()?)),
      ParticleActionType::Source => Self::Source(Box::new(reader.read_xr::<T, _>()?)),
      ParticleActionType::SpeedLimit => Self::SpeedLimit(Box::new(reader.read_xr::<T, _>()?)),
      ParticleActionType::TargetColor => Self::TargetColor(Box::new(reader.read_xr::<T, _>()?)),
      ParticleActionType::TargetSize => Self::TargetSize(Box::new(reader.read_xr::<T, _>()?)),
      ParticleActionType::TargetRotate | ParticleActionType::TargetRotateD => {
        Self::TargetRotate(Box::new(reader.read_xr::<T, _>()?))
      }
      ParticleActionType::TargetVelocity | ParticleActionType::TargetVelocityD => {
        Self::TargetVelocity(Box::new(reader.read_xr::<T, _>()?))
      }
      ParticleActionType::Vortex => Self::Vortex(Box::new(reader.read_xr::<T, _>()?)),
      ParticleActionType::Turbulence => Self::Turbulence(Box::new(reader.read_xr::<T, _>()?)),
      ParticleActionType::Scatter => Self::Scatter(Box::new(reader.read_xr::<T, _>()?)),
      ParticleActionType::Unknown | ParticleActionType::CallActionList => {
        return Err(XrfError::new_unexpected_error(format!(
          "Unexpected action type provided for reading: {}",
          action_type
        )));
      }
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    // TargetRotate/TargetVelocity share variants with their D-derivatives, so the
    // dispatch type must come from the stored action type to survive round-trip.
    let action_type: ParticleActionType = match self {
      ParticleAction::TargetRotate(action) => action.action_type,
      ParticleAction::TargetVelocity(action) => action.action_type,
      action => ParticleActionType::get_action_type(action),
    };

    writer.write_u32::<T>(action_type as u32)?;

    match self {
      ParticleAction::Avoid(action) => writer.write_xr::<T, _>(action.deref()),
      ParticleAction::Bounce(action) => writer.write_xr::<T, _>(action.deref()),
      ParticleAction::CopyVertex(action) => writer.write_xr::<T, _>(action.deref()),
      ParticleAction::Damping(action) => writer.write_xr::<T, _>(action.deref()),
      ParticleAction::Explosion(action) => writer.write_xr::<T, _>(action.deref()),
      ParticleAction::Follow(action) => writer.write_xr::<T, _>(action.deref()),
      ParticleAction::Gravitate(action) => writer.write_xr::<T, _>(action.deref()),
      ParticleAction::Gravity(action) => writer.write_xr::<T, _>(action.deref()),
      ParticleAction::Jet(action) => writer.write_xr::<T, _>(action.deref()),
      ParticleAction::KillOld(action) => writer.write_xr::<T, _>(action.deref()),
      ParticleAction::MatchVelocity(action) => writer.write_xr::<T, _>(action.deref()),
      ParticleAction::Move(action) => writer.write_xr::<T, _>(action.deref()),
      ParticleAction::OrbitLine(action) => writer.write_xr::<T, _>(action.deref()),
      ParticleAction::OrbitPoint(action) => writer.write_xr::<T, _>(action.deref()),
      ParticleAction::RandomAccel(action) => writer.write_xr::<T, _>(action.deref()),
      ParticleAction::RandomDisplace(action) => writer.write_xr::<T, _>(action.deref()),
      ParticleAction::RandomVelocity(action) => writer.write_xr::<T, _>(action.deref()),
      ParticleAction::Restore(action) => writer.write_xr::<T, _>(action.deref()),
      ParticleAction::Sink(action) => writer.write_xr::<T, _>(action.deref()),
      ParticleAction::SinkVelocity(action) => writer.write_xr::<T, _>(action.deref()),
      ParticleAction::Source(action) => writer.write_xr::<T, _>(action.deref()),
      ParticleAction::SpeedLimit(action) => writer.write_xr::<T, _>(action.deref()),
      ParticleAction::TargetColor(action) => writer.write_xr::<T, _>(action.deref()),
      ParticleAction::TargetSize(action) => writer.write_xr::<T, _>(action.deref()),
      ParticleAction::TargetRotate(action) => writer.write_xr::<T, _>(action.deref()),
      ParticleAction::TargetVelocity(action) => writer.write_xr::<T, _>(action.deref()),
      ParticleAction::Vortex(action) => writer.write_xr::<T, _>(action.deref()),
      ParticleAction::Turbulence(action) => writer.write_xr::<T, _>(action.deref()),
      ParticleAction::Scatter(action) => writer.write_xr::<T, _>(action.deref()),
    }
  }
}

impl LtxImportExport for ParticleAction {
  fn import(section_name: &str, ltx: &Ltx) -> XrfResult<Self> {
    let section: &Section = ltx.section(section_name).ok_or_else(|| {
      XrfError::new_parsing_error(format!(
        "Particle action section '{}' should be defined in ltx file ({})",
        section_name,
        file!()
      ))
    })?;

    let meta_type: String = read_ltx_field(META_TYPE_FIELD, section)?;

    assert_equal(
      meta_type.as_str(),
      Self::META_TYPE,
      "Expected corrected meta type field for particle action import",
    )?;

    let action_type: ParticleActionType =
      ParticleActionType::from_str(read_ltx_field::<String>("action_type", section)?.as_str())
        .map_err(|_| XrfError::new_parsing_error("Failed to parse particle action type from LTX field"))?;

    Ok(match action_type {
      ParticleActionType::Avoid => Self::Avoid(Box::new(ParticleActionAvoid::import(section_name, ltx)?)),
      ParticleActionType::Bounce => Self::Bounce(Box::new(ParticleActionBounce::import(section_name, ltx)?)),
      ParticleActionType::CopyVertex => {
        Self::CopyVertex(Box::new(ParticleActionCopyVertex::import(section_name, ltx)?))
      }
      ParticleActionType::Damping => Self::Damping(Box::new(ParticleActionDamping::import(section_name, ltx)?)),
      ParticleActionType::Explosion => Self::Explosion(Box::new(ParticleActionExplosion::import(section_name, ltx)?)),
      ParticleActionType::Follow => Self::Follow(Box::new(ParticleActionFollow::import(section_name, ltx)?)),
      ParticleActionType::Gravitate => Self::Gravitate(Box::new(ParticleActionGravitate::import(section_name, ltx)?)),
      ParticleActionType::Gravity => Self::Gravity(Box::new(ParticleActionGravity::import(section_name, ltx)?)),
      ParticleActionType::Jet => Self::Jet(Box::new(ParticleActionJet::import(section_name, ltx)?)),
      ParticleActionType::KillOld => Self::KillOld(Box::new(ParticleActionKillOld::import(section_name, ltx)?)),
      ParticleActionType::MatchVelocity => {
        Self::MatchVelocity(Box::new(ParticleActionMatchVelocity::import(section_name, ltx)?))
      }
      ParticleActionType::Move => Self::Move(Box::new(ParticleActionMove::import(section_name, ltx)?)),
      ParticleActionType::OrbitLine => Self::OrbitLine(Box::new(ParticleActionOrbitLine::import(section_name, ltx)?)),
      ParticleActionType::OrbitPoint => {
        Self::OrbitPoint(Box::new(ParticleActionOrbitPoint::import(section_name, ltx)?))
      }
      ParticleActionType::RandomAccel => {
        Self::RandomAccel(Box::new(ParticleActionRandomAcceleration::import(section_name, ltx)?))
      }
      ParticleActionType::RandomDisplace => {
        Self::RandomDisplace(Box::new(ParticleActionRandomDisplace::import(section_name, ltx)?))
      }
      ParticleActionType::RandomVelocity => {
        Self::RandomVelocity(Box::new(ParticleActionRandomVelocity::import(section_name, ltx)?))
      }
      ParticleActionType::Restore => Self::Restore(Box::new(ParticleActionRestore::import(section_name, ltx)?)),
      ParticleActionType::Sink => Self::Sink(Box::new(ParticleActionSink::import(section_name, ltx)?)),
      ParticleActionType::SinkVelocity => {
        Self::SinkVelocity(Box::new(ParticleActionSinkVelocity::import(section_name, ltx)?))
      }
      ParticleActionType::Source => Self::Source(Box::new(ParticleActionSource::import(section_name, ltx)?)),
      ParticleActionType::SpeedLimit => {
        Self::SpeedLimit(Box::new(ParticleActionSpeedLimit::import(section_name, ltx)?))
      }
      ParticleActionType::TargetColor => {
        Self::TargetColor(Box::new(ParticleActionTargetColor::import(section_name, ltx)?))
      }
      ParticleActionType::TargetSize => {
        Self::TargetSize(Box::new(ParticleActionTargetSize::import(section_name, ltx)?))
      }
      ParticleActionType::TargetRotate | ParticleActionType::TargetRotateD => {
        Self::TargetRotate(Box::new(ParticleActionTargetRotate::import(section_name, ltx)?))
      }
      ParticleActionType::TargetVelocity | ParticleActionType::TargetVelocityD => {
        Self::TargetVelocity(Box::new(ParticleActionTargetVelocity::import(section_name, ltx)?))
      }
      ParticleActionType::Vortex => Self::Vortex(Box::new(ParticleActionVortex::import(section_name, ltx)?)),
      ParticleActionType::Turbulence => {
        Self::Turbulence(Box::new(ParticleActionTurbulence::import(section_name, ltx)?))
      }
      ParticleActionType::Scatter => Self::Scatter(Box::new(ParticleActionScatter::import(section_name, ltx)?)),
      ParticleActionType::Unknown | ParticleActionType::CallActionList => {
        return Err(XrfError::new_unexpected_error(format!(
          "Unexpected action type provided for reading: {}",
          action_type
        )));
      }
    })
  }

  fn export(&self, section_name: &str, ltx: &mut Ltx) -> XrfResult {
    ltx.with_section(section_name).set(META_TYPE_FIELD, Self::META_TYPE);

    match self {
      ParticleAction::Avoid(action) => action.export(section_name, ltx),
      ParticleAction::Bounce(action) => action.export(section_name, ltx),
      ParticleAction::CopyVertex(action) => action.export(section_name, ltx),
      ParticleAction::Damping(action) => action.export(section_name, ltx),
      ParticleAction::Explosion(action) => action.export(section_name, ltx),
      ParticleAction::Follow(action) => action.export(section_name, ltx),
      ParticleAction::Gravitate(action) => action.export(section_name, ltx),
      ParticleAction::Gravity(action) => action.export(section_name, ltx),
      ParticleAction::Jet(action) => action.export(section_name, ltx),
      ParticleAction::KillOld(action) => action.export(section_name, ltx),
      ParticleAction::MatchVelocity(action) => action.export(section_name, ltx),
      ParticleAction::Move(action) => action.export(section_name, ltx),
      ParticleAction::OrbitLine(action) => action.export(section_name, ltx),
      ParticleAction::OrbitPoint(action) => action.export(section_name, ltx),
      ParticleAction::RandomAccel(action) => action.export(section_name, ltx),
      ParticleAction::RandomDisplace(action) => action.export(section_name, ltx),
      ParticleAction::RandomVelocity(action) => action.export(section_name, ltx),
      ParticleAction::Restore(action) => action.export(section_name, ltx),
      ParticleAction::Sink(action) => action.export(section_name, ltx),
      ParticleAction::SinkVelocity(action) => action.export(section_name, ltx),
      ParticleAction::Source(action) => action.export(section_name, ltx),
      ParticleAction::SpeedLimit(action) => action.export(section_name, ltx),
      ParticleAction::TargetColor(action) => action.export(section_name, ltx),
      ParticleAction::TargetSize(action) => action.export(section_name, ltx),
      ParticleAction::TargetRotate(action) => action.export(section_name, ltx),
      ParticleAction::TargetVelocity(action) => action.export(section_name, ltx),
      ParticleAction::Vortex(action) => action.export(section_name, ltx),
      ParticleAction::Turbulence(action) => action.export(section_name, ltx),
      ParticleAction::Scatter(action) => action.export(section_name, ltx),
    }
  }
}

#[cfg(test)]
mod tests {
  use byteorder::ReadBytesExt;
  use xrf_chunk::{
    ChunkReadWrite, ChunkReadWriteList, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder,
  };
  use xrf_error::XrfResult;
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::utils::{
    build_relative_test_sample_file_path, open_generated_test_resource_as_slice,
    overwrite_generated_test_resource_as_file,
  };

  use crate::data::generic::vector_3d::Vector3d;
  use crate::data::particles::actions::particle_action_target_rotate::ParticleActionTargetRotate;
  use crate::data::particles::actions::particle_action_target_velocity::ParticleActionTargetVelocity;
  use crate::data::particles::particle_action::ParticleAction;
  use crate::data::particles::particle_action_type::ParticleActionType;

  #[test]
  fn test_read_write_derivative_action_types() -> XrfResult {
    let mut writer: ChunkWriter = ChunkWriter::new();
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write_derivative.chunk");

    // Derivative types (TargetRotateD / TargetVelocityD) share enum variants with their
    // base types, so the dispatch u32 must round-trip from the stored action type.
    let rotate: ParticleAction = ParticleAction::TargetRotate(Box::new(ParticleActionTargetRotate {
      action_flags: 1,
      action_type: ParticleActionType::TargetRotateD,
      rot: Vector3d::new_mock(),
      scale: 1.5,
    }));
    let velocity: ParticleAction = ParticleAction::TargetVelocity(Box::new(ParticleActionTargetVelocity {
      action_flags: 1,
      action_type: ParticleActionType::TargetVelocityD,
      velocity: Vector3d::new_mock(),
      scale: 0.5,
    }));

    rotate.write::<XRayByteOrder>(&mut writer)?;
    velocity.write::<XRayByteOrder>(&mut writer)?;

    writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(&filename)?, 0)?;

    let file: FileSlice = open_generated_test_resource_as_slice(&filename)?;
    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    assert_eq!(
      reader.read_u32::<XRayByteOrder>()?,
      ParticleActionType::TargetRotateD as u32,
      "Expected derivative dispatch type to be written for target rotate"
    );

    let mut reader: ChunkReader =
      ChunkReader::from_slice(open_generated_test_resource_as_slice(&filename)?)?.read_child_by_index(0)?;

    assert_eq!(ParticleAction::read::<XRayByteOrder, _>(&mut reader)?, rotate);
    assert_eq!(ParticleAction::read::<XRayByteOrder, _>(&mut reader)?, velocity);

    Ok(())
  }

  #[test]
  fn rejects_action_count_larger_than_the_chunk_before_reserving_it() -> XrfResult {
    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[255, 255, 255, 255])?;

    let error: String = ParticleAction::read_list::<XRayByteOrder, _>(&mut reader)
      .expect_err("expect the declared action count to exceed the chunk")
      .to_string();

    assert!(
      error.contains("particle actions declares 4294967295 entries"),
      "Unexpected error: {error}"
    );

    Ok(())
  }
}
