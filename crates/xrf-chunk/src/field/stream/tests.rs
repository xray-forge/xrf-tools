use std::collections::VecDeque;

use byteorder::ByteOrder;
use xrf_error::{XrfError, XrfResult};

use crate::{
  ChunkField, ChunkFieldKind, ChunkFieldReadWrite, ChunkFieldReader, ChunkFieldRecorder, ChunkFieldReplay,
  ChunkFieldSource, ChunkFieldWriter, ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder,
};

/// Fields handed out in order whatever kind is asked, so the replay's own check of the kind is what a test sees.
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

/// A value of every primitive, closed by the size of its body patched in ahead of it, as `Spawn_Write` does.
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
  fn read_fields<T: ByteOrder, R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Self> {
    reader.read_u16::<T>()?;

    Ok(Self {
      flag: reader.read_u8()?,
      id: reader.read_u16::<T>()?,
      count: reader.read_u32::<T>()?,
      stamp: reader.read_u64::<T>()?,
      scale: reader.read_f32::<T>()?,
      at: reader.read_vector3::<T>()?,
      name: reader.read_w1251_string()?,
      ids: reader.read_u16_vector::<T>()?,
    })
  }

  fn write_fields<T: ByteOrder, W: ChunkFieldWriter>(&self, writer: &mut W) -> XrfResult {
    let opened: usize = writer.get_position();

    writer.write_u16::<T>(0)?;
    writer.write_u8(self.flag)?;
    writer.write_u16::<T>(self.id)?;
    writer.write_u32::<T>(self.count)?;
    writer.write_u64::<T>(self.stamp)?;
    writer.write_f32::<T>(self.scale)?;
    writer.write_vector3::<T>(self.at)?;
    writer.write_w1251_string(&self.name)?;
    writer.write_u16_vector::<T>(&self.ids)?;
    writer.patch_u16::<T>(opened, (writer.get_position() - opened) as u16)
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

/// One write serves both forms: the bytes read back through `ChunkReadWrite`, and the fields record each value with its
/// kind, the patched size kept as written, at the position the bytes reach.
#[test]
fn writes_bytes_and_fields_from_one_write() -> XrfResult {
  let sample: Sample = new_sample();
  let mut bytes: ChunkWriter = ChunkWriter::new();
  let mut recorder: ChunkFieldRecorder = ChunkFieldRecorder::new();

  sample.write::<XRayByteOrder>(&mut bytes)?;
  sample.write_fields::<XRayByteOrder, _>(&mut recorder)?;

  assert!(!bytes.is_field_form());
  assert!(recorder.is_field_form());
  assert_eq!(recorder.get_position(), bytes.bytes_written());
  assert_eq!(
    XRayByteOrder::read_u16(&bytes.buffer[..2]) as usize,
    bytes.bytes_written()
  );
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

  assert_eq!(Sample::read::<XRayByteOrder, _>(&mut reader)?, sample);
  assert!(reader.is_ended());

  Ok(())
}

/// The fields read back into the same value, the replay counting the bytes they stand for.
#[test]
fn replays_recorded_fields() -> XrfResult {
  let sample: Sample = new_sample();
  let mut recorder: ChunkFieldRecorder = ChunkFieldRecorder::new();

  sample.write_fields::<XRayByteOrder, _>(&mut recorder)?;

  let position: usize = recorder.get_position();
  let mut source: QueuedFields = QueuedFields(recorder.into_fields().into());
  let mut replay: ChunkFieldReplay<'_, QueuedFields> = ChunkFieldReplay::new(&mut source);

  assert_eq!(Sample::read_fields::<XRayByteOrder, _>(&mut replay)?, sample);
  assert_eq!(replay.get_position(), position);
  assert!(source.0.is_empty());

  Ok(())
}

#[test]
fn refuses_a_field_of_another_kind() {
  let mut source: QueuedFields = QueuedFields(VecDeque::from([ChunkField::U8(1)]));
  let error: String = Sample::read_fields::<XRayByteOrder, _>(&mut ChunkFieldReplay::new(&mut source))
    .expect_err("a u8 where the size's u16 stands")
    .to_string();

  assert!(error.contains("A U8 field stands where a U16 is read"), "{error}");
}

/// Raw bytes make no field: none is nothing in either direction, any is refused.
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
