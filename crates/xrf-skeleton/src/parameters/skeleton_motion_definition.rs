use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{assert_length, to_format_size};

use crate::parameters::{SkeletonMotionFlag, SkeletonMotionMark};

/// One motion's definition in the parameters chunk: its name, flags, target, the four floats it plays with, and marks.
#[derive(Clone, Debug, PartialEq)]
pub struct SkeletonMotionDefinition {
  pub name: String,
  pub flags: u32,
  pub bone_or_part: u16,
  pub motion: u16,
  pub speed: f32,
  pub power: f32,
  pub accrue: f32,
  pub falloff: f32,
  pub marks: Vec<SkeletonMotionMark>,
}

impl SkeletonMotionDefinition {
  /// The terminator of an empty name, the flags, the bone or part and motion ids, and the four float parameters. The
  /// marks that version 4 appends only make a record longer.
  pub const MIN_SERIALIZED_SIZE: u64 = 1 + 4 + 2 + 2 + 16;

  /// The first parameters version whose definitions carry marks, which `CMotionDef::Load` reads from it on.
  pub const MARKS_VERSION: u16 = 4;

  /// The rate a motion plays at when its own is unusable, which is the rate the format samples at.
  pub const DEFAULT_SPEED: f32 = 1.0;

  /// Steps per unit `CMotionDef::Quantize` keeps a stored float in, as a `u16` (`SkeletonMotions.hpp:146`).
  pub const QUANTUM: f32 = 655.35;

  /// What the engine widens a blend time by before blending over it, `fQuantizerRangeExt` (`SkeletonMotions.hpp:132`).
  pub const BLEND_RANGE_EXTENSION: f32 = 1.5;

  /// `CMotionDef::Quantize`: the steps a stored float floors to, clamped to a `u16`, which is all `CMotionDef::Load`
  /// keeps of speed, power, accrue and falloff.
  pub fn quantize(value: f32) -> u16 {
    // Clamped first, so the cast narrows nothing; a NaN, which `iFloor` leaves unusable, casts to zero.
    (value * Self::QUANTUM).floor().clamp(0.0, f32::from(u16::MAX)) as u16
  }

  /// `CMotionDef::Dequantize`: what a step count is worth.
  pub fn dequantize(steps: u16) -> f32 {
    f32::from(steps) / Self::QUANTUM
  }

  /// The largest value quantising keeps, about 100; anything past it, or below zero, is clamped.
  pub fn get_quantized_maximum() -> f32 {
    Self::dequantize(u16::MAX)
  }

  /// The speed the engine plays, `CMotionDef::Speed`.
  pub fn get_engine_speed(&self) -> f32 {
    Self::dequantize(Self::quantize(self.speed))
  }

  /// The power the engine blends with, `CMotionDef::Power`.
  pub fn get_engine_power(&self) -> f32 {
    Self::dequantize(Self::quantize(self.power))
  }

  /// What the engine blends in over, `CMotionDef::Accrue`.
  pub fn get_engine_accrue(&self) -> f32 {
    Self::BLEND_RANGE_EXTENSION * Self::dequantize(Self::quantize(self.accrue))
  }

  /// What the engine blends out over, `CMotionDef::Falloff`, after `CMotionDef::Load` replaces a cycle's falloff that
  /// quantises at or above its accrue with one step under it.
  pub fn get_engine_falloff(&self) -> f32 {
    let falloff: u16 = if self.is_falloff_replaced() {
      // `u16(accrue - 1)`, which wraps a cycle declaring no accrue round to the quantizer's ceiling.
      Self::quantize(self.accrue).wrapping_sub(1)
    } else {
      Self::quantize(self.falloff)
    };

    Self::BLEND_RANGE_EXTENSION * Self::dequantize(falloff)
  }

  /// Whether `CMotionDef::Load` replaces the declared falloff rather than reading it.
  pub fn is_falloff_replaced(&self) -> bool {
    !SkeletonMotionFlag::Fx.is_set_in(self.flags) && Self::quantize(self.falloff) >= Self::quantize(self.accrue)
  }

  /// The rate playback runs at, as a multiplier of the format's sample rate.
  ///
  /// This is the speed the engine plays ([`Self::get_engine_speed`]) rather than the one stored. A speed that comes to
  /// zero would never advance the motion, so it plays at [`Self::DEFAULT_SPEED`] instead.
  pub fn get_playback_speed(&self) -> f32 {
    match Self::quantize(self.speed) {
      0 => Self::DEFAULT_SPEED,
      steps => Self::dequantize(steps),
    }
  }

  pub fn read_list<T: ByteOrder, D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
    version: u16,
  ) -> XrfResult<Vec<Self>> {
    let count: u16 = reader.read_u16::<T>()?;
    let mut definitions: Vec<Self> =
      reader.new_bounded_vec(count.into(), Self::MIN_SERIALIZED_SIZE, "skeleton motion definitions")?;

    for _ in 0..count {
      definitions.push(
        Self::read::<T, _>(reader, version)
          .map_err(|error| XrfError::new_read_error(format!("Failed to read skeleton motion: {error}")))?,
      );
    }

    assert_length(
      &definitions,
      count as usize,
      "Expected correct count of skeleton motions to be read",
    )?;

    Ok(definitions)
  }

  pub fn read<T: ByteOrder, D: ChunkDataSource>(reader: &mut ChunkReader<D>, version: u16) -> XrfResult<Self> {
    let name: String = reader.read_w1251_string()?;
    let flags: u32 = reader.read_u32::<T>()?;
    let bone_or_part: u16 = reader.read_u16::<T>()?;
    let motion: u16 = reader.read_u16::<T>()?;
    let speed: f32 = reader.read_f32::<T>()?;
    let power: f32 = reader.read_f32::<T>()?;
    let accrue: f32 = reader.read_f32::<T>()?;
    let falloff: f32 = reader.read_f32::<T>()?;

    let marks: Vec<SkeletonMotionMark> = if version >= Self::MARKS_VERSION {
      let count: u32 = reader.read_u32::<T>()?;
      let mut marks: Vec<SkeletonMotionMark> = reader.new_bounded_vec(
        count.into(),
        SkeletonMotionMark::MIN_SERIALIZED_SIZE,
        "skeleton motion marks",
      )?;

      for _ in 0..count {
        marks.push(
          SkeletonMotionMark::read::<T, _>(reader)
            .map_err(|error| XrfError::new_read_error(format!("Failed to read skeleton motion mark: {error}")))?,
        );
      }

      assert_length(
        &marks,
        count as usize,
        "Expected correct count of skeleton motion marks to be read",
      )?;

      marks
    } else {
      Vec::new()
    };

    let motion: Self = Self {
      name,
      flags,
      bone_or_part,
      motion,
      speed,
      power,
      accrue,
      falloff,
      marks,
    };

    Ok(motion)
  }

  pub fn write_list<T: ByteOrder>(writer: &mut ChunkWriter, definitions: &[Self], version: u16) -> XrfResult {
    writer.write_u16::<T>(to_format_size(definitions.len(), "skeleton motion definitions")?)?;

    for definition in definitions {
      definition
        .write::<T>(writer, version)
        .map_err(|error| XrfError::new_serialization_error(format!("Failed to write skeleton motion: {error}")))?;
    }

    Ok(())
  }

  pub fn write<T: ByteOrder>(&self, writer: &mut ChunkWriter, version: u16) -> XrfResult {
    writer.write_w1251_string(&self.name)?;
    writer.write_u32::<T>(self.flags)?;
    writer.write_u16::<T>(self.bone_or_part)?;
    writer.write_u16::<T>(self.motion)?;
    writer.write_f32::<T>(self.speed)?;
    writer.write_f32::<T>(self.power)?;
    writer.write_f32::<T>(self.accrue)?;
    writer.write_f32::<T>(self.falloff)?;

    if version >= Self::MARKS_VERSION {
      writer.write_u32::<T>(to_format_size(self.marks.len(), "skeleton motion marks")?)?;

      for mark in &self.marks {
        mark.write::<T>(writer).map_err(|error| {
          XrfError::new_serialization_error(format!("Failed to write skeleton motion mark: {error}"))
        })?;
      }
    } else if !self.marks.is_empty() {
      return Err(XrfError::new_invalid_error(format!(
        "Cannot write {} skeleton motion marks for '{}', marks are only supported from version {}, got {version}",
        self.marks.len(),
        self.name,
        Self::MARKS_VERSION
      )));
    }

    Ok(())
  }
}

impl SkeletonMotionDefinition {
  #[cfg(any(test, feature = "fixtures"))]
  /// The definition this crate's own tests and its dependants' fixtures build against.
  pub fn new_mock(marks: Vec<SkeletonMotionMark>) -> Self {
    Self {
      name: String::from("ak74_draw"),
      flags: 3,
      bone_or_part: 1,
      motion: 7,
      speed: 1.0,
      power: 0.5,
      accrue: 2.0,
      falloff: 4.0,
      marks,
    }
  }
}

#[cfg(test)]
mod tests {
  use xrf_chunk::{ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
  use xrf_error::XrfResult;
  use xrf_test_utils::{FileSlice, GeneratedResources};

  use crate::parameters::{SkeletonMotionDefinition, SkeletonMotionFlag, SkeletonMotionMark};

  fn write_read_list(
    filename: &str,
    definitions: &[SkeletonMotionDefinition],
    version: u16,
  ) -> XrfResult<Vec<SkeletonMotionDefinition>> {
    let mut writer: ChunkWriter = ChunkWriter::new();

    SkeletonMotionDefinition::write_list::<XRayByteOrder>(&mut writer, definitions, version)?;

    writer.flush_chunk_into::<XRayByteOrder>(&mut GeneratedResources::overwrite(filename)?, 0)?;

    let file: FileSlice = GeneratedResources::open_slice(filename)?;
    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    SkeletonMotionDefinition::read_list::<XRayByteOrder, _>(&mut reader, version)
  }

  #[test]
  fn reports_the_speed_the_engine_quantises_a_stored_one_to() {
    let speed = |stored: f32| {
      let mut definition: SkeletonMotionDefinition = SkeletonMotionDefinition::new_mock(Vec::new());

      definition.speed = stored;
      definition.get_playback_speed()
    };

    // `Quantize` floors: 1.0 is 655.35 steps, kept as 655.
    assert_eq!(speed(1.0), 655.0 / 655.35);
    assert_eq!(speed(1.2), 786.0 / 655.35);
    // A `u16` of steps tops out at 100.
    assert_eq!(speed(250.0), 65535.0 / 655.35);
  }

  #[test]
  fn quantises_as_the_engine_does_without_wrapping() {
    assert_eq!(SkeletonMotionDefinition::quantize(1.0), 655);
    assert_eq!(SkeletonMotionDefinition::quantize(150.0), u16::MAX);
    assert_eq!(SkeletonMotionDefinition::quantize(-1.0), 0);
    assert_eq!(SkeletonMotionDefinition::quantize(f32::NAN), 0);
    assert_eq!(SkeletonMotionDefinition::get_quantized_maximum(), 65535.0 / 655.35);
  }

  #[test]
  fn blends_over_half_again_what_the_definition_declares() {
    let mut definition: SkeletonMotionDefinition = SkeletonMotionDefinition::new_mock(Vec::new());

    // A cycle whose falloff quantises under its accrue keeps it.
    (definition.flags, definition.accrue, definition.falloff) = (0, 2.0, 1.0);

    assert_eq!(definition.get_engine_accrue(), 1.5 * (1310.0 / 655.35));
    assert_eq!(definition.get_engine_falloff(), 1.5 * (655.0 / 655.35));
    assert!(!definition.is_falloff_replaced());
  }

  #[test]
  fn replaces_a_cycles_falloff_at_or_above_its_accrue_but_not_an_effects() {
    let mut definition: SkeletonMotionDefinition = SkeletonMotionDefinition::new_mock(Vec::new());

    (definition.flags, definition.accrue, definition.falloff) = (0, 2.0, 5.0);

    assert!(definition.is_falloff_replaced());
    assert_eq!(definition.get_engine_falloff(), 1.5 * (1309.0 / 655.35));

    definition.flags = SkeletonMotionFlag::Fx.mask();

    assert!(!definition.is_falloff_replaced());
    assert_eq!(definition.get_engine_falloff(), 1.5 * (3276.0 / 655.35));
  }

  #[test]
  fn wraps_a_cycle_declaring_no_accrue_to_the_quantizers_ceiling() {
    // `u16(0 - 1)` is 65535, not 0: such a cycle blends out over the longest time the engine can hold.
    let mut definition: SkeletonMotionDefinition = SkeletonMotionDefinition::new_mock(Vec::new());

    (definition.flags, definition.accrue, definition.falloff) = (0, 0.0, 0.0);

    assert!(definition.is_falloff_replaced());
    assert_eq!(definition.get_engine_accrue(), 0.0);
    assert_eq!(definition.get_engine_falloff(), 1.5 * (65535.0 / 655.35));
  }

  #[test]
  fn reads_a_speed_that_cannot_scale_playback_as_the_sample_rate() {
    // Each quantises to zero steps, which never advances the motion and leaves a length nothing can be divided into.
    for speed in [0.0, -1.0, f32::NAN, 0.001] {
      let mut definition: SkeletonMotionDefinition = SkeletonMotionDefinition::new_mock(Vec::new());

      definition.speed = speed;

      assert_eq!(
        definition.get_playback_speed(),
        SkeletonMotionDefinition::DEFAULT_SPEED,
        "expect {speed} to be read as the default speed"
      );
    }
  }

  #[test]
  fn rejects_a_definition_list_past_its_count_field() {
    let definitions: Vec<SkeletonMotionDefinition> = vec![SkeletonMotionDefinition::new_mock(Vec::new()); 65536];
    let mut writer: ChunkWriter = ChunkWriter::new();

    assert_eq!(
      SkeletonMotionDefinition::write_list::<XRayByteOrder>(&mut writer, &definitions, 4)
        .expect_err("expect the definition count to exceed its format field")
        .to_string(),
      "Invalid error: skeleton motion definitions exceeds the u16 format limit"
    );
  }

  #[test]
  fn test_read_write_list_v4_with_marks() -> XrfResult {
    let filename: String = GeneratedResources::sample_name(file!(), "read_write_v4.chunk");

    let original: Vec<SkeletonMotionDefinition> = vec![
      SkeletonMotionDefinition::new_mock(vec![SkeletonMotionMark {
        name: String::from("Left"),
        terminator: String::from("\r\n"),
        intervals: vec![(0.1, 0.2)],
      }]),
      SkeletonMotionDefinition::new_mock(Vec::new()),
    ];

    assert_eq!(write_read_list(&filename, &original, 4)?, original);

    Ok(())
  }

  #[test]
  fn test_read_write_list_v3_without_marks() -> XrfResult {
    let filename: String = GeneratedResources::sample_name(file!(), "read_write_v3.chunk");

    let original: Vec<SkeletonMotionDefinition> = vec![SkeletonMotionDefinition::new_mock(Vec::new())];

    assert_eq!(write_read_list(&filename, &original, 3)?, original);

    Ok(())
  }

  #[test]
  fn test_write_v3_with_marks_is_rejected() {
    let mut writer: ChunkWriter = ChunkWriter::new();

    let definition: SkeletonMotionDefinition = SkeletonMotionDefinition::new_mock(vec![SkeletonMotionMark {
      name: String::from("Left"),
      terminator: String::from("\r\n"),
      intervals: vec![(0.1, 0.2)],
    }]);

    assert!(
      definition.write::<XRayByteOrder>(&mut writer, 3).is_err(),
      "Expect marks to be rejected when writing version 3 motion definition"
    );
  }

  #[test]
  fn rejects_definition_count_larger_than_the_chunk_before_reserving_it() -> XrfResult {
    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[255, 255])?;

    let error: String = SkeletonMotionDefinition::read_list::<XRayByteOrder, _>(&mut reader, 4)
      .expect_err("expect the declared definition count to exceed the chunk")
      .to_string();

    assert!(
      error.contains("skeleton motion definitions declares 65535 entries"),
      "Unexpected error: {error}"
    );

    Ok(())
  }

  #[test]
  fn rejects_mark_count_larger_than_the_chunk_before_reserving_it() -> XrfResult {
    let mut bytes: Vec<u8> = vec![0];

    // Flags, the bone or part and motion ids, and the four float parameters, all zeroed.
    bytes.extend([0; 4 + 2 + 2 + 16]);
    bytes.extend([255, 255, 255, 255]);

    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&bytes)?;

    let error: String = SkeletonMotionDefinition::read::<XRayByteOrder, _>(&mut reader, 4)
      .expect_err("expect the declared mark count to exceed the chunk")
      .to_string();

    assert!(
      error.contains("skeleton motion marks declares 4294967295 entries"),
      "Unexpected error: {error}"
    );

    Ok(())
  }
}
