use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use derive_more::{Display, FromStr};
use enum_map::Enum;
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::data::particle_action::ParticleAction;

#[derive(Copy, Clone, Debug, Enum, PartialEq, FromStr, Display, Serialize, Deserialize)]
pub enum ParticleActionType {
  #[display("Avoid")]
  Avoid = 0,
  #[display("Bounce")]
  Bounce = 1,
  #[display("CallActionList")]
  CallActionList = 2,
  #[display("CopyVertex")]
  CopyVertex = 3,
  #[display("Damping")]
  Damping = 4,
  #[display("Explosion")]
  Explosion = 5,
  #[display("Follow")]
  Follow = 6,
  #[display("Gravitate")]
  Gravitate = 7,
  #[display("Gravity")]
  Gravity = 8,
  #[display("Jet")]
  Jet = 9,
  #[display("KillOld")]
  KillOld = 10,
  #[display("MatchVelocity")]
  MatchVelocity = 11,
  #[display("Move")]
  Move = 12,
  #[display("OrbitLine")]
  OrbitLine = 13,
  #[display("OrbitPoint")]
  OrbitPoint = 14,
  #[display("RandomAccel")]
  RandomAccel = 15,
  #[display("RandomDisplace")]
  RandomDisplace = 16,
  #[display("RandomVelocity")]
  RandomVelocity = 17,
  #[display("Restore")]
  Restore = 18,
  #[display("Sink")]
  Sink = 19,
  #[display("SinkVelocity")]
  SinkVelocity = 20,
  #[display("Source")]
  Source = 21,
  #[display("SpeedLimit")]
  SpeedLimit = 22,
  #[display("TargetColor")]
  TargetColor = 23,
  #[display("TargetSize")]
  TargetSize = 24,
  #[display("TargetRotate")]
  TargetRotate = 25,
  #[display("TargetRotateD")]
  TargetRotateD = 26,
  #[display("TargetVelocity")]
  TargetVelocity = 27,
  #[display("TargetVelocityD")]
  TargetVelocityD = 28,
  #[display("Vortex")]
  Vortex = 29,
  #[display("Turbulence")]
  Turbulence = 30,
  #[display("Scatter")]
  Scatter = 31,
  #[display("Unknown")]
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
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self::from(reader.read_u32::<T>()?))
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(*self as u32)?;

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

  use crate::data::particle_action_type::ParticleActionType;

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
