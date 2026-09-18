use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

use crate::level::level_light_color::LevelLightColor;

/// One static light of a compiled level, as `fsL_LIGHT_DYNAMIC` stores it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelDynamicLight {
  /// The light animation controlling this light, which the renderer reads past without using.
  pub controller: u32,
  /// `Flight::Type`: point, spot or directional.
  pub kind: u32,
  pub diffuse: LevelLightColor,
  /// Written by the compiler and overwritten on load, which takes it from the diffuse at a fifth of the brightness.
  pub specular: LevelLightColor,
  pub ambient: LevelLightColor,
  pub position: Vector3d,
  pub direction: Vector3d,
  pub range: f32,
  pub falloff: f32,
  pub attenuation_0: f32,
  pub attenuation_1: f32,
  pub attenuation_2: f32,
  /// The inner angle of a spotlight cone.
  pub theta: f32,
  /// The outer angle of a spotlight cone.
  pub phi: f32,
}

impl LevelDynamicLight {
  /// `Flight::Type::Point`.
  pub const KIND_POINT: u32 = 1;

  /// `Flight::Type::Spot`.
  pub const KIND_SPOT: u32 = 2;

  /// `Flight::Type::Directional`, which the loader keeps as the level's sun.
  pub const KIND_DIRECTIONAL: u32 = 3;

  /// Bytes one record occupies: the controller, then `Flight`.
  pub const SERIALIZED_SIZE: u64 = 4 + 4 + (3 * LevelLightColor::SERIALIZED_SIZE) + (2 * 12) + (7 * 4);

  /// The engine's name for the kind, or a labelled unknown for a value `Flight::Type` does not name.
  pub fn get_kind_label(&self) -> String {
    match self.kind {
      Self::KIND_POINT => String::from("point"),
      Self::KIND_SPOT => String::from("spot"),
      Self::KIND_DIRECTIONAL => String::from("directional"),
      unknown => format!("unknown({unknown})"),
    }
  }

  /// Whether this light is the one the loader keeps as the level's sun.
  pub const fn is_sun(&self) -> bool {
    self.kind == Self::KIND_DIRECTIONAL
  }
}

impl ChunkReadWrite for LevelDynamicLight {
  fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      controller: reader.read_u32::<T>()?,
      kind: reader.read_u32::<T>()?,
      diffuse: reader.read_xr::<T, _>()?,
      specular: reader.read_xr::<T, _>()?,
      ambient: reader.read_xr::<T, _>()?,
      position: reader.read_xr::<T, _>()?,
      direction: reader.read_xr::<T, _>()?,
      range: reader.read_f32::<T>()?,
      falloff: reader.read_f32::<T>()?,
      attenuation_0: reader.read_f32::<T>()?,
      attenuation_1: reader.read_f32::<T>()?,
      attenuation_2: reader.read_f32::<T>()?,
      theta: reader.read_f32::<T>()?,
      phi: reader.read_f32::<T>()?,
    })
  }

  fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.controller)?;
    writer.write_u32::<T>(self.kind)?;

    self.diffuse.write::<T>(writer)?;
    self.specular.write::<T>(writer)?;
    self.ambient.write::<T>(writer)?;
    self.position.write::<T>(writer)?;
    self.direction.write::<T>(writer)?;

    for value in [
      self.range,
      self.falloff,
      self.attenuation_0,
      self.attenuation_1,
      self.attenuation_2,
      self.theta,
      self.phi,
    ] {
      writer.write_f32::<T>(value)?;
    }

    Ok(())
  }
}
