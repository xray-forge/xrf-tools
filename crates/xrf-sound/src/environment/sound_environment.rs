use std::ops::RangeInclusive;

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};

use crate::environment::eax_environment::EaxEnvironment;

/// One reverb preset, `CSoundRender_Environment` (`xrSound/SoundRender_Environment.cpp`): the EAX 2 listener
/// properties the engine sets while the listener is in a space the level marks with it. By default the preset
/// `CSoundRender_Environment`'s constructor makes, the EAX defaults under no name.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoundEnvironment {
  /// The name a level's sound environments reach the preset by.
  pub name: String,
  /// Room effect level at low frequencies, in hundredths of a decibel.
  pub room: f32,
  /// Room effect level at high frequencies relative to low ones, in hundredths of a decibel.
  pub room_hf: f32,
  /// How the room effect falls off with distance, as DirectSound's rolloff does for the direct sound.
  pub room_rolloff_factor: f32,
  /// Seconds the reverberation takes to decay at low frequencies.
  pub decay_time: f32,
  /// How much faster or slower high frequencies decay than low ones.
  pub decay_hf_ratio: f32,
  /// Early reflections level relative to the room effect, in hundredths of a decibel.
  pub reflections: f32,
  /// Seconds before the first reflection.
  pub reflections_delay: f32,
  /// Late reverberation level relative to the room effect, in hundredths of a decibel.
  pub reverb: f32,
  /// Seconds from the first reflection to the late reverberation.
  pub reverb_delay: f32,
  /// Metres across the space sounds.
  pub environment_size: f32,
  /// Echo density in the reverberation decay, from none to full.
  pub environment_diffusion: f32,
  /// Hundredths of a decibel lost per metre at 5 kHz.
  pub air_absorption_hf: f32,
  /// The room the preset was authored from, which only a version 4 preset stores. A version 3 one reads as none, and
  /// the version written follows from it alone: see [`Self::get_version`].
  pub environment: Option<EaxEnvironment>,
}

impl SoundEnvironment {
  /// The version below which `CSoundRender_Environment::load` skips a record rather than read it.
  pub const MINIMUM_VERSION: u32 = 3;

  /// The version from which a record carries its EAX room, and the one `save` writes, `sdef_env_version`.
  pub const ENVIRONMENT_VERSION: u32 = 4;

  /// Versions this reads and writes back.
  pub const SUPPORTED_VERSIONS: [u32; 2] = [Self::MINIMUM_VERSION, Self::ENVIRONMENT_VERSION];

  pub const ROOM_RANGE: RangeInclusive<f32> = -10000.0..=0.0;
  pub const ROOM_HF_RANGE: RangeInclusive<f32> = -10000.0..=0.0;
  pub const ROOM_ROLLOFF_FACTOR_RANGE: RangeInclusive<f32> = 0.0..=10.0;
  pub const DECAY_TIME_RANGE: RangeInclusive<f32> = 0.1..=20.0;
  pub const DECAY_HF_RATIO_RANGE: RangeInclusive<f32> = 0.1..=2.0;
  pub const REFLECTIONS_RANGE: RangeInclusive<f32> = -10000.0..=1000.0;
  pub const REFLECTIONS_DELAY_RANGE: RangeInclusive<f32> = 0.0..=0.3;
  pub const REVERB_RANGE: RangeInclusive<f32> = -10000.0..=2000.0;
  pub const REVERB_DELAY_RANGE: RangeInclusive<f32> = 0.0..=0.1;
  pub const ENVIRONMENT_SIZE_RANGE: RangeInclusive<f32> = 1.0..=100.0;
  pub const ENVIRONMENT_DIFFUSION_RANGE: RangeInclusive<f32> = 0.0..=1.0;
  pub const AIR_ABSORPTION_HF_RANGE: RangeInclusive<f32> = -100.0..=0.0;

  /// A preset of the EAX defaults under `name`, `SoundEnvironment_LIB::Append` without a parent.
  pub fn new(name: &str) -> Self {
    Self {
      name: name.to_owned(),
      ..Self::default()
    }
  }

  /// Reads one preset from the chunk holding it, `CSoundRender_Environment::load`.
  ///
  /// The engine reads any version from 3 on, a later one as version 4. One past 4 is refused here, since writing it
  /// back would change its version; no shipped library has one.
  ///
  /// # Errors
  ///
  /// Returns an error when the chunk does not hold a preset of a version this reads.
  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let version: u32 = reader.read_u32::<T>()?;

    if !Self::SUPPORTED_VERSIONS.contains(&version) {
      return Err(XrfError::new_not_implemented_error(format!(
        "Unexpected sound environment version {version} on read, only versions {:?} are implemented",
        Self::SUPPORTED_VERSIONS
      )));
    }

    Ok(Self {
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
        Some(EaxEnvironment::from(reader.read_u32::<T>()?))
      } else {
        None
      },
    })
  }

  /// Writes the preset back in its own version, which keeps a version 3 preset version 3: the engine's own `save`
  /// always writes version 4, see [`Self::normalize`].
  ///
  /// # Errors
  ///
  /// Returns an error when the writer refuses the bytes.
  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter) -> XrfResult {
    writer.write_u32::<T>(self.get_version())?;
    writer.write_w1251_string(&self.name)?;

    for value in self.get_parameters() {
      writer.write_f32::<T>(value)?;
    }

    if let Some(environment) = self.environment {
      writer.write_u32::<T>(environment.into())?;
    }

    Ok(())
  }

  /// The version the preset is: version 4 when it stores its room, and version 3 otherwise.
  pub const fn get_version(&self) -> u32 {
    if self.environment.is_some() {
      Self::ENVIRONMENT_VERSION
    } else {
      Self::MINIMUM_VERSION
    }
  }

  /// Turns the preset into what `CSoundRender_Environment::save` writes: version 4, a version 3 preset taking the
  /// room the engine left it at, [`EaxEnvironment::Generic`].
  pub fn normalize(&mut self) {
    self.environment.get_or_insert_default();
  }

  /// `CSoundRender_Environment::set_default`: the EAX listener defaults and the generic room, the name kept.
  pub fn set_default(&mut self) {
    *self = Self {
      name: std::mem::take(&mut self.name),
      ..Self::default()
    };
  }

  /// `CSoundRender_Environment::set_identity`, the SDK's "Identity" button: the defaults with the room effect silenced,
  /// which adds no reverb at all. The engine plays it wherever a level marks no environment.
  pub fn set_identity(&mut self) {
    self.set_default();
    self.room = *Self::ROOM_RANGE.start();
    self.clamp();
  }

  /// `CSoundRender_Environment::clamp`: every parameter within the range EAX accepts it in, which are also the ranges
  /// the SDK's environment editor offers.
  pub fn clamp(&mut self) {
    let ranges: [RangeInclusive<f32>; 12] = Self::get_parameter_ranges();

    for (value, range) in self.get_parameters_mut().into_iter().zip(ranges) {
      *value = value.clamp(*range.start(), *range.end());
    }
  }

  /// `CSoundRender_Environment::lerp`: this preset's parameters a `factor` of the way to those of `to`, then clamped,
  /// under this preset's name and room. The engine moves the listener's and each emitter's environment towards the one
  /// they are in this way, by the frame's time each frame (`e_current.lerp(e_current, e_target, dt)`).
  pub fn lerp(&self, to: &Self, factor: f32) -> Self {
    let mut blended: Self = self.clone();

    for (value, to) in blended.get_parameters_mut().into_iter().zip(to.get_parameters()) {
      *value = (1.0 - factor) * *value + factor * to;
    }

    blended.clamp();
    blended
  }

  /// The twelve reverb parameters, in the order the file stores them.
  pub const fn get_parameters(&self) -> [f32; 12] {
    [
      self.room,
      self.room_hf,
      self.room_rolloff_factor,
      self.decay_time,
      self.decay_hf_ratio,
      self.reflections,
      self.reflections_delay,
      self.reverb,
      self.reverb_delay,
      self.environment_size,
      self.environment_diffusion,
      self.air_absorption_hf,
    ]
  }

  /// The range EAX accepts each of [`Self::get_parameters`] in, in the same order.
  pub const fn get_parameter_ranges() -> [RangeInclusive<f32>; 12] {
    [
      Self::ROOM_RANGE,
      Self::ROOM_HF_RANGE,
      Self::ROOM_ROLLOFF_FACTOR_RANGE,
      Self::DECAY_TIME_RANGE,
      Self::DECAY_HF_RATIO_RANGE,
      Self::REFLECTIONS_RANGE,
      Self::REFLECTIONS_DELAY_RANGE,
      Self::REVERB_RANGE,
      Self::REVERB_DELAY_RANGE,
      Self::ENVIRONMENT_SIZE_RANGE,
      Self::ENVIRONMENT_DIFFUSION_RANGE,
      Self::AIR_ABSORPTION_HF_RANGE,
    ]
  }

  fn get_parameters_mut(&mut self) -> [&mut f32; 12] {
    [
      &mut self.room,
      &mut self.room_hf,
      &mut self.room_rolloff_factor,
      &mut self.decay_time,
      &mut self.decay_hf_ratio,
      &mut self.reflections,
      &mut self.reflections_delay,
      &mut self.reverb,
      &mut self.reverb_delay,
      &mut self.environment_size,
      &mut self.environment_diffusion,
      &mut self.air_absorption_hf,
    ]
  }
}

impl Default for SoundEnvironment {
  /// `CSoundRender_Environment`'s constructor: version 4 and `set_default`'s EAX listener defaults
  /// (`EAXLISTENER_DEFAULT*`), under no name.
  fn default() -> Self {
    Self {
      name: String::new(),
      room: -1000.0,
      room_hf: -100.0,
      room_rolloff_factor: 0.0,
      decay_time: 1.49,
      decay_hf_ratio: 0.83,
      reflections: -2602.0,
      reflections_delay: 0.007,
      reverb: 200.0,
      reverb_delay: 0.011,
      environment_size: 7.5,
      environment_diffusion: 1.0,
      air_absorption_hf: -5.0,
      environment: Some(EaxEnvironment::default()),
    }
  }
}
