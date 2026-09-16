use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};

/// One reverb preset, `CSoundRender_Environment` (`xrSound/SoundRender_Environment.cpp`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoundEnvironment {
  pub version: u32,
  pub name: String,
  pub room: f32,
  pub room_hf: f32,
  pub room_rolloff_factor: f32,
  pub decay_time: f32,
  pub decay_hf_ratio: f32,
  pub reflections: f32,
  pub reflections_delay: f32,
  pub reverb: f32,
  pub reverb_delay: f32,
  pub environment_size: f32,
  pub environment_diffusion: f32,
  pub air_absorption_hf: f32,
  /// The EAX preset the environment stands for, which only version 4 and above declares.
  pub environment: Option<u32>,
}

impl SoundEnvironment {
  /// The lowest version `CSoundRender_Environment::load` accepts; below it the record is skipped, not refused.
  pub const MINIMUM_VERSION: u32 = 3;

  /// The version from which a record carries its own EAX preset number.
  pub const ENVIRONMENT_VERSION: u32 = 4;

  /// Reads one preset from the chunk holding it.
  ///
  /// # Errors
  ///
  /// Returns an error when the chunk does not hold a preset of a version the engine reads.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let version: u32 = reader.read_u32::<T>()?;

    if version < Self::MINIMUM_VERSION {
      return Err(XrfError::new_not_implemented_error(format!(
        "Unexpected sound environment version {version} on read, the engine reads {} and above",
        Self::MINIMUM_VERSION
      )));
    }

    Ok(Self {
      version,
      name: reader.read_w1251_string()?,
      room: reader.read_f32::<T>()?,
      room_hf: reader.read_f32::<T>()?,
      room_rolloff_factor: reader.read_f32::<T>()?,
      decay_time: reader.read_f32::<T>()?,
      decay_hf_ratio: reader.read_f32::<T>()?,
      reflections: reader.read_f32::<T>()?,
      reflections_delay: reader.read_f32::<T>()?,
      reverb: reader.read_f32::<T>()?,
      reverb_delay: reader.read_f32::<T>()?,
      environment_size: reader.read_f32::<T>()?,
      environment_diffusion: reader.read_f32::<T>()?,
      air_absorption_hf: reader.read_f32::<T>()?,
      environment: if version >= Self::ENVIRONMENT_VERSION {
        Some(reader.read_u32::<T>()?)
      } else {
        None
      },
    })
  }

  /// Writes the preset back in the layout `CSoundRender_Environment::save` writes it in.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer refuses the bytes.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.version)?;
    writer.write_w1251_string(&self.name)?;
    writer.write_f32::<T>(self.room)?;
    writer.write_f32::<T>(self.room_hf)?;
    writer.write_f32::<T>(self.room_rolloff_factor)?;
    writer.write_f32::<T>(self.decay_time)?;
    writer.write_f32::<T>(self.decay_hf_ratio)?;
    writer.write_f32::<T>(self.reflections)?;
    writer.write_f32::<T>(self.reflections_delay)?;
    writer.write_f32::<T>(self.reverb)?;
    writer.write_f32::<T>(self.reverb_delay)?;
    writer.write_f32::<T>(self.environment_size)?;
    writer.write_f32::<T>(self.environment_diffusion)?;
    writer.write_f32::<T>(self.air_absorption_hf)?;

    if let Some(environment) = self.environment {
      writer.write_u32::<T>(environment)?;
    }

    Ok(())
  }
}
