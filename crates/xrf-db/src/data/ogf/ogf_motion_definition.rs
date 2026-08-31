use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt};
use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, ChunkWriter};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{assert_length, to_format_size};

use crate::data::ogf::ogf_motion_mark::OgfMotionMark;

#[derive(Clone, Debug, PartialEq)]
pub struct OgfMotionDefinition {
  pub name: String,
  pub flags: u32,
  pub bone_or_part: u16,
  pub motion: u16,
  pub speed: f32,
  pub power: f32,
  pub accrue: f32,
  pub falloff: f32,
  pub marks: Vec<OgfMotionMark>,
}

// todo: Version based switcher?
// todo: Version based switcher?
// todo: Version based switcher?
impl OgfMotionDefinition {
  /// The terminator of an empty name, the flags, the bone or part and motion ids, and the four float parameters. The
  /// marks that version 4 appends only make a record longer.
  pub const MIN_SERIALIZED_SIZE: u64 = 1 + 4 + 2 + 2 + 16;

  /// The rate a motion plays at when its own is unusable, which is the rate the format samples at.
  pub const DEFAULT_SPEED: f32 = 1.0;

  /// The rate playback runs at, as a multiplier of the format's sample rate.
  ///
  /// The engine divides a motion's length by this (`SkeletonAnimated.cpp:290`), so it scales how long the motion takes
  /// rather than how many frames it has. A stored value that is not positive - zero, negative, or not a number -
  /// would make that division infinite or backwards, so it reads as [`Self::DEFAULT_SPEED`]; the stored value stays
  /// untouched, since nothing here writes it back.
  pub fn get_playback_speed(&self) -> f32 {
    if self.speed > 0.0 {
      self.speed
    } else {
      Self::DEFAULT_SPEED
    }
  }

  pub fn read_list<T: ByteOrder, D: ChunkDataSource>(
    reader: &mut ChunkReader<D>,
    version: u16,
  ) -> XrfResult<Vec<Self>> {
    let count: u16 = reader.read_u16::<T>()?;
    let mut definitions: Vec<Self> =
      reader.new_bounded_vec(count.into(), Self::MIN_SERIALIZED_SIZE, "ogf motion definitions")?;

    for _ in 0..count {
      definitions.push(
        Self::read::<T, _>(reader, version)
          .map_err(|error| XrfError::new_read_error(format!("Failed to read ogf motion: {error}")))?,
      );
    }

    assert_length(
      &definitions,
      count as usize,
      "Expected correct count of OGF motions to be read",
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

    let marks: Vec<OgfMotionMark> = if version == 4 {
      let count: u32 = reader.read_u32::<T>()?;
      let mut marks: Vec<OgfMotionMark> =
        reader.new_bounded_vec(count.into(), OgfMotionMark::MIN_SERIALIZED_SIZE, "ogf motion marks")?;

      for _ in 0..count {
        marks.push(
          OgfMotionMark::read::<T, _>(reader)
            .map_err(|error| XrfError::new_read_error(format!("Failed to read ogf motion mark: {error}")))?,
        );
      }

      assert_length(
        &marks,
        count as usize,
        "Expected correct count of OGF motion marks to be read",
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
    writer.write_u16::<T>(to_format_size(definitions.len(), "ogf motion definitions")?)?;

    for definition in definitions {
      definition
        .write::<T>(writer, version)
        .map_err(|error| XrfError::new_serialization_error(format!("Failed to write ogf motion: {error}")))?;
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

    if version == 4 {
      writer.write_u32::<T>(to_format_size(self.marks.len(), "ogf motion marks")?)?;

      for mark in &self.marks {
        mark
          .write::<T>(writer)
          .map_err(|error| XrfError::new_serialization_error(format!("Failed to write ogf motion mark: {error}")))?;
      }
    } else if !self.marks.is_empty() {
      return Err(XrfError::new_invalid_error(format!(
        "Cannot write {} ogf motion marks for '{}', marks are only supported in version 4, got {version}",
        self.marks.len(),
        self.name
      )));
    }

    Ok(())
  }
}

impl OgfMotionDefinition {
  #[cfg(test)]
  pub fn new_mock(marks: Vec<OgfMotionMark>) -> Self {
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
  use xrf_test_utils::FileSlice;
  use xrf_test_utils::utils::{
    build_relative_test_sample_file_path, open_generated_test_resource_as_slice,
    overwrite_generated_test_resource_as_file,
  };

  use crate::data::ogf::ogf_motion_definition::OgfMotionDefinition;
  use crate::data::ogf::ogf_motion_mark::OgfMotionMark;

  fn write_read_list(
    filename: &str,
    definitions: &[OgfMotionDefinition],
    version: u16,
  ) -> XrfResult<Vec<OgfMotionDefinition>> {
    let mut writer: ChunkWriter = ChunkWriter::new();

    OgfMotionDefinition::write_list::<XRayByteOrder>(&mut writer, definitions, version)?;

    writer.flush_chunk_into::<XRayByteOrder>(&mut overwrite_generated_test_resource_as_file(filename)?, 0)?;

    let file: FileSlice = open_generated_test_resource_as_slice(filename)?;
    let mut reader: ChunkReader = ChunkReader::from_slice(file)?.read_child_by_index(0)?;

    OgfMotionDefinition::read_list::<XRayByteOrder, _>(&mut reader, version)
  }

  #[test]
  fn reports_a_stored_playback_speed_as_it_is() {
    let mut definition: OgfMotionDefinition = OgfMotionDefinition::new_mock(Vec::new());

    definition.speed = 1.2;

    assert_eq!(definition.get_playback_speed(), 1.2);
  }

  #[test]
  fn reads_a_speed_that_cannot_scale_playback_as_the_sample_rate() {
    // The engine divides a motion's length by its speed, which these three turn into infinity, a negative length, and
    // a value nothing can be compared against.
    for speed in [0.0, -1.0, f32::NAN] {
      let mut definition: OgfMotionDefinition = OgfMotionDefinition::new_mock(Vec::new());

      definition.speed = speed;

      assert_eq!(
        definition.get_playback_speed(),
        OgfMotionDefinition::DEFAULT_SPEED,
        "expect {speed} to be read as the default speed"
      );
    }
  }

  #[test]
  fn rejects_a_definition_list_past_its_count_field() {
    let definitions: Vec<OgfMotionDefinition> = vec![OgfMotionDefinition::new_mock(Vec::new()); 65536];
    let mut writer: ChunkWriter = ChunkWriter::new();

    assert_eq!(
      OgfMotionDefinition::write_list::<XRayByteOrder>(&mut writer, &definitions, 4)
        .expect_err("expect the definition count to exceed its format field")
        .to_string(),
      "Invalid error: ogf motion definitions exceeds the u16 format limit"
    );
  }

  #[test]
  fn test_read_write_list_v4_with_marks() -> XrfResult {
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write_v4.chunk");

    let original: Vec<OgfMotionDefinition> = vec![
      OgfMotionDefinition::new_mock(vec![OgfMotionMark {
        name: String::from("Left"),
        terminator: String::from("\r\n"),
        intervals: vec![(0.1, 0.2)],
      }]),
      OgfMotionDefinition::new_mock(Vec::new()),
    ];

    assert_eq!(write_read_list(&filename, &original, 4)?, original);

    Ok(())
  }

  #[test]
  fn test_read_write_list_v3_without_marks() -> XrfResult {
    let filename: String = build_relative_test_sample_file_path(file!(), "read_write_v3.chunk");

    let original: Vec<OgfMotionDefinition> = vec![OgfMotionDefinition::new_mock(Vec::new())];

    assert_eq!(write_read_list(&filename, &original, 3)?, original);

    Ok(())
  }

  #[test]
  fn test_write_v3_with_marks_is_rejected() {
    let mut writer: ChunkWriter = ChunkWriter::new();

    let definition: OgfMotionDefinition = OgfMotionDefinition::new_mock(vec![OgfMotionMark {
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

    let error: String = OgfMotionDefinition::read_list::<XRayByteOrder, _>(&mut reader, 4)
      .expect_err("expect the declared definition count to exceed the chunk")
      .to_string();

    assert!(
      error.contains("ogf motion definitions declares 65535 entries"),
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

    let error: String = OgfMotionDefinition::read::<XRayByteOrder, _>(&mut reader, 4)
      .expect_err("expect the declared mark count to exceed the chunk")
      .to_string();

    assert!(
      error.contains("ogf motion marks declares 4294967295 entries"),
      "Unexpected error: {error}"
    );

    Ok(())
  }
}
