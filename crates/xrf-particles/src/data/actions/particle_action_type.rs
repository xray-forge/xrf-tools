use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
use xrf_enum_derive::{EnumDisplay, EnumFromStr};
use xrf_error::XrfResult;

use crate::data::actions::particle_action::ParticleAction;

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumFromStr, EnumDisplay, Serialize, Deserialize)]
pub enum ParticleActionType {
  Avoid = 0,
  Bounce = 1,
  CallActionList = 2,
  CopyVertex = 3,
  Damping = 4,
  Explosion = 5,
  Follow = 6,
  Gravitate = 7,
  Gravity = 8,
  Jet = 9,
  KillOld = 10,
  MatchVelocity = 11,
  Move = 12,
  OrbitLine = 13,
  OrbitPoint = 14,
  RandomAccel = 15,
  RandomDisplace = 16,
  RandomVelocity = 17,
  Restore = 18,
  Sink = 19,
  SinkVelocity = 20,
  Source = 21,
  SpeedLimit = 22,
  TargetColor = 23,
  TargetSize = 24,
  TargetRotate = 25,
  TargetRotateD = 26,
  TargetVelocity = 27,
  TargetVelocityD = 28,
  Vortex = 29,
  Turbulence = 30,
  Scatter = 31,
  Unknown = -1,
}

impl ParticleActionType {
  pub fn get_action_type(action: &ParticleAction) -> Self {
    match action {
      ParticleAction::Avoid(_) => Self::Avoid,
      ParticleAction::Bounce(_) => Self::Bounce,
      ParticleAction::CopyVertex(_) => Self::CopyVertex,
      ParticleAction::Damping(_) => Self::Damping,
      ParticleAction::Explosion(_) => Self::Explosion,
      ParticleAction::Follow(_) => Self::Follow,
      ParticleAction::Gravitate(_) => Self::Gravitate,
      ParticleAction::Gravity(_) => Self::Gravity,
      ParticleAction::Jet(_) => Self::Jet,
      ParticleAction::KillOld(_) => Self::KillOld,
      ParticleAction::MatchVelocity(_) => Self::MatchVelocity,
      ParticleAction::Move(_) => Self::Move,
      ParticleAction::OrbitLine(_) => Self::OrbitLine,
      ParticleAction::OrbitPoint(_) => Self::OrbitPoint,
      ParticleAction::RandomAccel(_) => Self::RandomAccel,
      ParticleAction::RandomDisplace(_) => Self::RandomDisplace,
      ParticleAction::RandomVelocity(_) => Self::RandomVelocity,
      ParticleAction::Restore(_) => Self::Restore,
      ParticleAction::Sink(_) => Self::Sink,
      ParticleAction::SinkVelocity(_) => Self::SinkVelocity,
      ParticleAction::Source(_) => Self::Source,
      ParticleAction::SpeedLimit(_) => Self::SpeedLimit,
      ParticleAction::TargetColor(_) => Self::TargetColor,
      ParticleAction::TargetSize(_) => Self::TargetSize,
      ParticleAction::TargetRotate(_) => Self::TargetRotate,
      ParticleAction::TargetVelocity(_) => Self::TargetVelocity,
      ParticleAction::Vortex(_) => Self::Vortex,
      ParticleAction::Turbulence(_) => Self::Turbulence,
      ParticleAction::Scatter(_) => Self::Scatter,
    }
  }
}

impl ChunkReadWrite for ParticleActionType {
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self::from(reader.read_u32()?))
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32(*self as u32)?;

    Ok(())
  }
}

impl<T> From<T> for ParticleActionType
where
  T: Into<u32>,
{
  fn from(action_type: T) -> Self {
    match action_type.into() {
      0 => Self::Avoid,
      1 => Self::Bounce,
      2 => Self::CallActionList,
      3 => Self::CopyVertex,
      4 => Self::Damping,
      5 => Self::Explosion,
      6 => Self::Follow,
      7 => Self::Gravitate,
      8 => Self::Gravity,
      9 => Self::Jet,
      10 => Self::KillOld,
      11 => Self::MatchVelocity,
      12 => Self::Move,
      13 => Self::OrbitLine,
      14 => Self::OrbitPoint,
      15 => Self::RandomAccel,
      16 => Self::RandomDisplace,
      17 => Self::RandomVelocity,
      18 => Self::Restore,
      19 => Self::Sink,
      20 => Self::SinkVelocity,
      21 => Self::Source,
      22 => Self::SpeedLimit,
      23 => Self::TargetColor,
      24 => Self::TargetSize,
      25 => Self::TargetRotate,
      26 => Self::TargetRotateD,
      27 => Self::TargetVelocity,
      28 => Self::TargetVelocityD,
      29 => Self::Vortex,
      30 => Self::Turbulence,
      31 => Self::Scatter,
      _ => Self::Unknown,
    }
  }
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;

  use crate::data::actions::particle_action_type::ParticleActionType;

  #[test]
  fn test_from_str() {
    assert_eq!(
      ParticleActionType::from_str("KillOld").unwrap(),
      ParticleActionType::KillOld
    );
    assert_eq!(
      ParticleActionType::from_str("TargetRotateD").unwrap(),
      ParticleActionType::TargetRotateD
    );
    assert!(ParticleActionType::from_str("10").is_err());
    assert!(ParticleActionType::from_str("NotAnAction").is_err());
  }
}
