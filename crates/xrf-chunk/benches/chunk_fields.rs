//! What reading and writing values through a chunk costs: numbers, strings, typed records and the field form.

use std::collections::VecDeque;
use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use fileslice::FileSlice;
use xrf_chunk::{
  ChunkDataSource, ChunkField, ChunkFieldKind, ChunkFieldReadWrite, ChunkFieldReader, ChunkFieldRecorder,
  ChunkFieldReplay, ChunkFieldSource, ChunkFieldWriter, ChunkReadWrite, ChunkReader, ChunkWriter,
  InMemoryChunkDataSource, XRayBytes, XRayRead, XRayWrite,
};
use xrf_error::{XrfError, XrfResult};
use xrf_test_utils::GeneratedResources;

/// Vertices in the in-memory number groups: about 2.4 MB, a large OGF's vertex chunk.
const VERTEX_COUNT: usize = 65_536;

/// Vertices read through a file window, fewer because every number there is its own positional read.
const FILE_VERTEX_COUNT: usize = 4_096;

/// Bytes per vertex: position, normal, two texture coordinates and a bone index.
const VERTEX_SIZE: usize = 36;

/// Strings per string group, each followed by its terminator.
const STRING_COUNT: usize = 4_096;

/// String lengths before the terminator: a short id, a section name, and a long description or path.
const STRING_LENGTHS: [usize; 3] = [8, 32, 200];

/// Spawn-shaped records per record group, near a level's object count.
const SPAWN_RECORD_COUNT: usize = 4_096;

/// A skinned vertex as OGF stores it, read one number at a time.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Vertex {
  position: [f32; 3],
  normal: [f32; 3],
  uv: [f32; 2],
  bone: u32,
}

impl Vertex {
  fn new(index: usize) -> Self {
    let value: f32 = index as f32;

    Self {
      position: [value, value + 1.0, value + 2.0],
      normal: [0.0, 1.0, 0.0],
      uv: [value / 7.0, value / 11.0],
      bone: (index % 64) as u32,
    }
  }

  /// Decodes a vertex from bytes already in hand, as an in-place decoder does.
  fn from_bytes(bytes: &[u8; VERTEX_SIZE]) -> Self {
    let float = |at: usize| XRayBytes::read_f32(&bytes[at..]);

    Self {
      position: [float(0), float(4), float(8)],
      normal: [float(12), float(16), float(20)],
      uv: [float(24), float(28)],
      bone: XRayBytes::read_u32(&bytes[32..]),
    }
  }
}

impl ChunkReadWrite for Vertex {
  fn read<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let mut float = || XRayRead::read_f32(reader);

    Ok(Self {
      position: [float()?, float()?, float()?],
      normal: [float()?, float()?, float()?],
      uv: [float()?, float()?],
      bone: XRayRead::read_u32(reader)?,
    })
  }

  fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    for value in self.position.iter().chain(&self.normal).chain(&self.uv) {
      XRayWrite::write_f32(writer, *value)?;
    }

    Ok(XRayWrite::write_u32(writer, self.bone)?)
  }
}

/// An ALife object's leading fields, written once through the `NET_Packet` primitives as every spawn class is: a body
/// size patched in ahead of it, numbers, a position, a section name and a `u16` list.
#[derive(Clone, Debug, PartialEq)]
struct SpawnRecord {
  flags: u8,
  id: u16,
  game_vertex: u32,
  stamp: u64,
  angle: f32,
  position: [f32; 3],
  section: String,
  links: Vec<u16>,
}

impl SpawnRecord {
  fn new(index: usize) -> Self {
    Self {
      flags: (index % 8) as u8,
      id: (index % 65_535) as u16,
      game_vertex: index as u32,
      stamp: index as u64 * 1_000,
      angle: index as f32 / 3.0,
      position: [index as f32, 0.5, -(index as f32)],
      section: format!("zat_stalker_{index:05}"),
      links: vec![1, 2, 3, 4],
    }
  }
}

impl ChunkFieldReadWrite for SpawnRecord {
  fn read_fields<R: ChunkFieldReader>(reader: &mut R) -> XrfResult<Self> {
    reader.read_u16()?;

    Ok(Self {
      flags: reader.read_u8()?,
      id: reader.read_u16()?,
      game_vertex: reader.read_u32()?,
      stamp: reader.read_u64()?,
      angle: reader.read_f32()?,
      position: reader.read_vector3()?,
      section: reader.read_w1251_string()?,
      links: reader.read_u16_vector()?,
    })
  }

  fn write_fields<W: ChunkFieldWriter>(&self, writer: &mut W) -> XrfResult {
    let opened: usize = writer.get_position();

    writer.write_u16(0)?;
    writer.write_u8(self.flags)?;
    writer.write_u16(self.id)?;
    writer.write_u32(self.game_vertex)?;
    writer.write_u64(self.stamp)?;
    writer.write_f32(self.angle)?;
    writer.write_vector3(self.position)?;
    writer.write_w1251_string(&self.section)?;
    writer.write_u16_vector(&self.links)?;

    let size: u16 = u16::try_from(writer.get_position() - opened).expect("a spawn record under 64 KB");

    writer.patch_u16(opened, size)
  }
}

/// Fields queued in memory, standing in for an ini section the Level Editor keeps a spawn's data in.
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

fn vertices(count: usize) -> Vec<Vertex> {
  (0..count).map(Vertex::new).collect()
}

/// Writes records back to back with no count, the shape of a vertex or index payload.
fn write_vertices(writer: &mut ChunkWriter, vertices: &[Vertex]) {
  for vertex in vertices {
    writer.write_xr(vertex).expect("a vertex");
  }
}

fn vertex_bytes(vertices: &[Vertex]) -> Vec<u8> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  write_vertices(&mut writer, vertices);

  writer.buffer
}

fn spawn_records() -> Vec<SpawnRecord> {
  (0..SPAWN_RECORD_COUNT).map(SpawnRecord::new).collect()
}

/// `count` null-terminated strings of `length` bytes each.
fn string_bytes(count: usize, length: usize, fill: u8) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::with_capacity(count * (length + 1));

  for _ in 0..count {
    bytes.extend(std::iter::repeat_n(fill, length));
    bytes.push(0);
  }

  bytes
}

/// Reads every vertex one number at a time, as a field-by-field format reader does.
fn read_vertices_by_field<D: ChunkDataSource>(reader: &mut ChunkReader<D>, count: usize) -> Vec<Vertex> {
  reader.reset_pos().expect("a seekable source");

  (0..count)
    .map(|_| Vertex::read(reader).expect("a whole vertex"))
    .collect()
}

/// Decodes every vertex from bytes lent where they lie.
fn read_vertices_in_place(reader: &mut ChunkReader<InMemoryChunkDataSource>, count: usize) -> Vec<Vertex> {
  reader.reset_pos().expect("a seekable source");
  reader
    .read_with(count * VERTEX_SIZE, |bytes| {
      bytes
        .as_chunks::<VERTEX_SIZE>()
        .0
        .iter()
        .map(Vertex::from_bytes)
        .collect()
    })
    .expect("the whole vertex payload")
}

/// Numbers: the per-element cost of field-by-field reads against decoding in place, and over a file window.
fn bench_numbers(criterion: &mut Criterion) {
  let mut group = criterion.benchmark_group("chunk_fields/numbers");
  let expected: Vec<Vertex> = vertices(VERTEX_COUNT);
  let mut reader: ChunkReader<InMemoryChunkDataSource> =
    ChunkReader::from_vec(vertex_bytes(&expected)).expect("vertex bytes");

  assert_eq!(read_vertices_by_field(&mut reader, VERTEX_COUNT), expected);
  assert_eq!(read_vertices_in_place(&mut reader, VERTEX_COUNT), expected);

  group.throughput(Throughput::Elements(VERTEX_COUNT as u64));
  group.bench_function("read_by_field", |bencher| {
    bencher.iter(|| black_box(read_vertices_by_field(&mut reader, VERTEX_COUNT)));
  });
  group.bench_function("read_in_place", |bencher| {
    bencher.iter(|| black_box(read_vertices_in_place(&mut reader, VERTEX_COUNT)));
  });
  group.bench_function("read_records", |bencher| {
    bencher.iter(|| {
      reader.reset_pos().expect("a seekable source");
      black_box(
        reader
          .read_xr_records::<Vertex>(VERTEX_SIZE as u64, "vertices")
          .expect("whole records"),
      )
    });
  });
  group.bench_function("write_by_field", |bencher| {
    bencher.iter(|| {
      let mut writer: ChunkWriter = ChunkWriter::new();

      write_vertices(&mut writer, black_box(&expected));
      black_box(writer.buffer)
    });
  });

  // The reader's default source. Production readers load a file whole first (`from_file`), so this is the price of
  // reading field by field without doing that.
  let file_expected: Vec<Vertex> = vertices(FILE_VERTEX_COUNT);
  let path: String = GeneratedResources::sample_name(file!(), "vertices.bin");

  GeneratedResources::write(&path, vertex_bytes(&file_expected)).expect("a written vertex file");

  let mut file_reader: ChunkReader<FileSlice> =
    ChunkReader::from_slice(GeneratedResources::open_slice(&path).expect("the vertex file")).expect("a file window");

  assert_eq!(
    read_vertices_by_field(&mut file_reader, FILE_VERTEX_COUNT),
    file_expected
  );

  group.throughput(Throughput::Elements(FILE_VERTEX_COUNT as u64));
  group.bench_function("read_by_field_file", |bencher| {
    bencher.iter(|| black_box(read_vertices_by_field(&mut file_reader, FILE_VERTEX_COUNT)));
  });

  group.finish();
}

/// The `u32` count and `u16` run that index lists and spawn links store.
fn bench_u16_vector(criterion: &mut Criterion) {
  let mut group = criterion.benchmark_group("chunk_fields/u16_vector");
  let values: Vec<u16> = (0..VERTEX_COUNT).map(|index| (index % 65_535) as u16).collect();
  let mut writer: ChunkWriter = ChunkWriter::new();

  writer.write_u16_vector(&values).expect("a u16 vector");

  let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_vec(writer.buffer).expect("vector bytes");

  assert_eq!(reader.read_u16_vector().expect("the vector"), values);

  group.throughput(Throughput::Elements(values.len() as u64));
  group.bench_function("read", |bencher| {
    bencher.iter(|| {
      reader.reset_pos().expect("a seekable source");
      black_box(reader.read_u16_vector().expect("the vector"))
    });
  });
  group.bench_function("write", |bencher| {
    bencher.iter(|| {
      let mut writer: ChunkWriter = ChunkWriter::new();

      writer.write_u16_vector(black_box(&values)).expect("a u16 vector");
      black_box(writer.buffer)
    });
  });

  group.finish();
}

/// Null-terminated and line-terminated Windows-1251 strings, read and written.
fn bench_strings(criterion: &mut Criterion) {
  let mut group = criterion.benchmark_group("chunk_fields/strings");

  group.throughput(Throughput::Elements(STRING_COUNT as u64));

  for length in STRING_LENGTHS {
    let bytes: Vec<u8> = string_bytes(STRING_COUNT, length, b'a');
    let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_vec(bytes).expect("string bytes");

    reader.reset_pos().expect("a seekable source");
    assert_eq!(reader.read_w1251_string().expect("a string").len(), length);

    group.bench_with_input(BenchmarkId::new("read_ascii", length), &length, |bencher, _| {
      bencher.iter(|| {
        reader.reset_pos().expect("a seekable source");

        for _ in 0..STRING_COUNT {
          black_box(reader.read_w1251_string().expect("a string"));
        }
      });
    });
  }

  // Cyrillic takes the decoder off its ASCII path: every byte becomes a two-byte UTF-8 character.
  let length: usize = 32;
  let mut reader: ChunkReader<InMemoryChunkDataSource> =
    ChunkReader::from_vec(string_bytes(STRING_COUNT, length, 0xE0)).expect("string bytes");

  assert_eq!(reader.read_w1251_string().expect("a string").chars().count(), length);

  group.bench_with_input(BenchmarkId::new("read_cyrillic", length), &length, |bencher, _| {
    bencher.iter(|| {
      reader.reset_pos().expect("a seekable source");

      for _ in 0..STRING_COUNT {
        black_box(reader.read_w1251_string().expect("a string"));
      }
    });
  });

  let lines: Vec<u8> = (0..STRING_COUNT)
    .flat_map(|_| [&[b'a'; 32][..], b"\r\n"].concat())
    .collect();
  let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_vec(lines).expect("line bytes");

  assert_eq!(reader.read_w1251_line().expect("a line").terminator, "\r\n");

  group.bench_with_input(BenchmarkId::new("read_line", length), &length, |bencher, _| {
    bencher.iter(|| {
      reader.reset_pos().expect("a seekable source");

      for _ in 0..STRING_COUNT {
        black_box(reader.read_w1251_line().expect("a line"));
      }
    });
  });

  for length in STRING_LENGTHS {
    let value: String = "a".repeat(length);

    group.bench_with_input(BenchmarkId::new("write_ascii", length), &value, |bencher, value| {
      bencher.iter(|| {
        let mut writer: ChunkWriter = ChunkWriter::new();

        for _ in 0..STRING_COUNT {
          writer.write_w1251_string(black_box(value)).expect("a string");
        }

        black_box(writer.buffer)
      });
    });
  }

  group.finish();
}

/// One spawn-shaped record type as bytes and as fields, the two forms every ALife class reads and writes in.
fn bench_spawn_records(criterion: &mut Criterion) {
  let mut group = criterion.benchmark_group("chunk_fields/spawn_records");
  let records: Vec<SpawnRecord> = spawn_records();
  let mut writer: ChunkWriter = ChunkWriter::new();

  for record in &records {
    writer.write_xr(record).expect("a spawn record");
  }

  let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_vec(writer.buffer).expect("record bytes");
  let read_bytes = |reader: &mut ChunkReader<InMemoryChunkDataSource>| -> Vec<SpawnRecord> {
    reader.reset_pos().expect("a seekable source");

    (0..SPAWN_RECORD_COUNT)
      .map(|_| reader.read_xr::<SpawnRecord>().expect("a spawn record"))
      .collect()
  };

  assert_eq!(read_bytes(&mut reader), records);

  let mut recorder: ChunkFieldRecorder = ChunkFieldRecorder::new();

  for record in &records {
    ChunkFieldWriter::write_xr(&mut recorder, record).expect("a recorded spawn record");
  }

  let fields: VecDeque<ChunkField> = recorder.into_fields().into();
  let replay = |fields: &VecDeque<ChunkField>| -> Vec<SpawnRecord> {
    let mut source: QueuedFields = QueuedFields(fields.clone());
    let mut replay: ChunkFieldReplay<'_, QueuedFields> = ChunkFieldReplay::new(&mut source);

    (0..SPAWN_RECORD_COUNT)
      .map(|_| ChunkFieldReader::read_xr::<SpawnRecord>(&mut replay).expect("a replayed spawn record"))
      .collect()
  };

  assert_eq!(replay(&fields), records);

  group.throughput(Throughput::Elements(SPAWN_RECORD_COUNT as u64));
  group.bench_function("read_bytes", |bencher| {
    bencher.iter(|| black_box(read_bytes(&mut reader)));
  });
  group.bench_function("write_bytes", |bencher| {
    bencher.iter(|| {
      let mut writer: ChunkWriter = ChunkWriter::new();

      for record in black_box(&records) {
        writer.write_xr(record).expect("a spawn record");
      }

      black_box(writer.buffer)
    });
  });
  // The queue is cloned inside the measurement because replaying consumes it, so this is an upper bound.
  group.bench_function("read_fields", |bencher| {
    bencher.iter(|| black_box(replay(black_box(&fields))));
  });
  group.bench_function("write_fields", |bencher| {
    bencher.iter(|| {
      let mut recorder: ChunkFieldRecorder = ChunkFieldRecorder::new();

      for record in black_box(&records) {
        ChunkFieldWriter::write_xr(&mut recorder, record).expect("a recorded spawn record");
      }

      black_box(recorder.into_fields())
    });
  });

  group.finish();
}

criterion_group!(
  benches,
  bench_numbers,
  bench_u16_vector,
  bench_strings,
  bench_spawn_records
);
criterion_main!(benches);
