use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter, XRayRead, XRayWrite};
use xrf_error::XrfResult;
use xrf_math::{Color, Vector3d};

/// One static light of a compiled level, as `fsL_LIGHT_DYNAMIC` stores it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelDynamicLight {
  /// The light animation controlling this light, which the renderer reads past without using.
  pub controller: u32,
  /// `Flight::Type`: point, spot or directional.
  pub kind: u32,
  pub diffuse: Color,
  /// Written by the compiler and overwritten on load, which takes it from the diffuse at a fifth of the brightness.
  pub specular: Color,
  pub ambient: Color,
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
  pub const SERIALIZED_SIZE: u64 = 4 + 4 + (3 * Color::SERIALIZED_SIZE) + (2 * 12) + (7 * 4);

  /// A directional light under `controller` of `diffuse` along `direction`, the rest of it zero, as the editor writes
  /// its generated sky and sun.
  pub fn new_directional(controller: u32, diffuse: Color, direction: Vector3d) -> Self {
    Self {
      controller,
      kind: Self::KIND_DIRECTIONAL,
      diffuse,
      specular: Color::new(0.0, 0.0, 0.0, 0.0),
      ambient: Color::new(0.0, 0.0, 0.0, 0.0),
      position: Vector3d::ZERO,
      direction,
      range: 0.0,
      falloff: 0.0,
      attenuation_0: 0.0,
      attenuation_1: 0.0,
      attenuation_2: 0.0,
      theta: 0.0,
      phi: 0.0,
    }
  }

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
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    Ok(Self {
      controller: reader.read_u32()?,
      kind: reader.read_u32()?,
      diffuse: reader.read_xr()?,
      specular: reader.read_xr()?,
      ambient: reader.read_xr()?,
      position: reader.read_xr()?,
      direction: reader.read_xr()?,
      range: reader.read_f32()?,
      falloff: reader.read_f32()?,
      attenuation_0: reader.read_f32()?,
      attenuation_1: reader.read_f32()?,
      attenuation_2: reader.read_f32()?,
      theta: reader.read_f32()?,
      phi: reader.read_f32()?,
    })
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32(self.controller)?;
    writer.write_u32(self.kind)?;

    self.diffuse.write(writer)?;
    self.specular.write(writer)?;
    self.ambient.write(writer)?;
    self.position.write(writer)?;
    self.direction.write(writer)?;

    for value in [
      self.range,
      self.falloff,
      self.attenuation_0,
      self.attenuation_1,
      self.attenuation_2,
      self.theta,
      self.phi,
    ] {
      writer.write_f32(value)?;
    }

    Ok(())
  }
}
