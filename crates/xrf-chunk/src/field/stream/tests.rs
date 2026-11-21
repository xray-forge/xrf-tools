use std::collections::VecDeque;

use xrf_error::{XrfError, XrfResult};

use crate::bytes::xray_bytes::XRayBytes;
use crate::field::chunk_field::ChunkField;
use crate::field::chunk_field_kind::ChunkFieldKind;
use crate::field::chunk_field_reader::ChunkFieldReader;
use crate::field::chunk_field_writer::ChunkFieldWriter;
use crate::field::stream::chunk_field_recorder::ChunkFieldRecorder;
use crate::field::stream::chunk_field_replay::ChunkFieldReplay;
use crate::field::stream::chunk_field_source::ChunkFieldSource;
use crate::read_write::chunk_field_read_write::ChunkFieldReadWrite;
use crate::read_write::chunk_read_write::ChunkReadWrite;
use crate::reader::chunk_reader::ChunkReader;
use crate::source::chunk_memory_source::InMemoryChunkDataSource;
use crate::writer::chunk_writer::ChunkWriter;

// Ignores the kind asked, so the replay's own kind check is what a test sees.
struct QueuedFields(VecDeque<ChunkField>);

impl ChunkFieldSource for QueuedFields {
  fn read_field(&mut self, _: ChunkFieldKind) -> XrfResult<ChunkField> {
    self
      .0
      .pop_front()
      .ok_or_else(|| XrfError::new_parsing_error("No field left"))
  }

  fn is_ended(&self) -> bool {
    self.0.is_empty()
  }
}

// Every primitive, with its body size patched in ahead of it as `Spawn_Write` does.
#[derive(Debug, PartialEq)]
struct Sample {
  flag: u8,
  id: u16,
  count: u32,
  stamp: u64,
  scale: f32,
  at: [f32; 3],
  name: String,
  ids: Vec<u16>,
}

impl ChunkFieldReadWrite for Sample {
  fn read_fields<R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Self> {
    reader.read_u16()?;

    Ok(Self {
      flag: reader.read_u8()?,
      id: reader.read_u16()?,
      count: reader.read_u32()?,
      stamp: reader.read_u64()?,
      scale: reader.read_f32()?,
      at: reader.read_vector3()?,
      name: reader.read_w1251_string()?,
      ids: reader.read_u16_vector()?,
    })
  }

  fn write_fields<W: ChunkFieldWriter>(&self, writer: &mut W) -> XrfResult {
    let opened: usize = writer.get_position();

    writer.write_u16(0)?;
    writer.write_u8(self.flag)?;
    writer.write_u16(self.id)?;
    writer.write_u32(self.count)?;
    writer.write_u64(self.stamp)?;
    writer.write_f32(self.scale)?;
    writer.write_vector3(self.at)?;
    writer.write_w1251_string(&self.name)?;
    writer.write_u16_vector(&self.ids)?;
    writer.patch_u16(opened, (writer.get_position() - opened) as u16)
  }
}

fn new_sample() -> Sample {
  Sample {
    flag: 254,
    id: u16::MAX,
    count: 0xFFFF_FF3E,
    stamp: u64::MAX - 1,
    scale: 0.5,
    at: [-55.5, 0.0, 93.25],
    name: String::from("Сталкер"),
    ids: vec![3, 7],
  }
}

// The fields keep the size as first written; only the bytes are patched.
#[test]
fn writes_bytes_and_fields_from_one_write() -> XrfResult {
  let sample: Sample = new_sample();
  let mut bytes: ChunkWriter = ChunkWriter::new();
  let mut recorder: ChunkFieldRecorder = ChunkFieldRecorder::new();

  sample.write(&mut bytes)?;
  sample.write_fields(&mut recorder)?;

  assert!(!bytes.is_field_form());
  assert!(recorder.is_field_form());
  assert_eq!(recorder.get_position(), bytes.bytes_written());
  assert_eq!(XRayBytes::read_u16(&bytes.buffer[..2]) as usize, bytes.bytes_written());
  assert_eq!(
    recorder.into_fields(),
    vec![
      ChunkField::U16(0),
      ChunkField::U8(254),
      ChunkField::U16(u16::MAX),
      ChunkField::U32(0xFFFF_FF3E),
      ChunkField::U64(u64::MAX - 1),
      ChunkField::F32(0.5),
      ChunkField::Vector3([-55.5, 0.0, 93.25]),
      ChunkField::String(String::from("Сталкер")),
      ChunkField::U32(2),
      ChunkField::U16(3),
      ChunkField::U16(7),
    ]
  );

  let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&bytes.buffer)?;

  assert_eq!(Sample::read(&mut reader)?, sample);
  assert!(reader.is_ended());

  Ok(())
}

#[test]
fn replays_recorded_fields() -> XrfResult {
  let sample: Sample = new_sample();
  let mut recorder: ChunkFieldRecorder = ChunkFieldRecorder::new();

  sample.write_fields(&mut recorder)?;

  let position: usize = recorder.get_position();
  let mut source: QueuedFields = QueuedFields(recorder.into_fields().into());
  let mut replay: ChunkFieldReplay<'_, QueuedFields> = ChunkFieldReplay::new(&mut source);

  assert_eq!(Sample::read_fields(&mut replay)?, sample);
  assert_eq!(replay.get_position(), position);
  assert!(source.0.is_empty());

  Ok(())
}

#[test]
fn refuses_a_field_of_another_kind() {
  let mut source: QueuedFields = QueuedFields(VecDeque::from([ChunkField::U8(1)]));
  let error: String = Sample::read_fields(&mut ChunkFieldReplay::new(&mut source))
    .expect_err("a u8 where the size's u16 stands")
    .to_string();

  assert!(error.contains("A U8 field stands where a U16 is read"), "{error}");
}

// No bytes pass in either direction; any are refused.
#[test]
fn refuses_raw_bytes_in_fields() -> XrfResult {
  let mut recorder: ChunkFieldRecorder = ChunkFieldRecorder::new();
  let mut source: QueuedFields = QueuedFields(VecDeque::new());
  let mut replay: ChunkFieldReplay<'_, QueuedFields> = ChunkFieldReplay::new(&mut source);

  recorder.write_bytes(&[])?;
  assert!(replay.read_bytes(0)?.is_empty());
  assert!(
    recorder
      .write_bytes(&[4, 8, 15])
      .expect_err("raw bytes")
      .to_string()
      .contains("Cannot record 3 raw bytes")
  );
  assert!(
    replay
      .read_bytes(3)
      .expect_err("raw bytes")
      .to_string()
      .contains("Cannot read 3 raw bytes")
  );
  assert_eq!(recorder.get_position(), 0);

  Ok(())
}
