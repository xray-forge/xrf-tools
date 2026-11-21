//! What the chunk reader costs per byte and per chunk.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use fileslice::FileSlice;
use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkSizePackedIterator, InMemoryChunkDataSource};
use xrf_test_utils::GeneratedResources;

/// Payload sizes for the bytes-dominated shape.
const PAYLOAD_SIZES: [usize; 3] = [4 * 1024, 256 * 1024, 4 * 1024 * 1024];

/// Chunk counts for the overhead-dominated shape, each carrying a body too small to matter.
const CHUNK_COUNTS: [usize; 3] = [64, 1024, 16384];

/// Bytes per chunk in the overhead-dominated shape.
const SMALL_PAYLOAD: usize = 16;

/// Builds a flat container of `count` chunks, each with `payload` bytes.
///
/// The on-disk layout is a `u32` id, a `u32` size, then the body — the same bytes a volume or a loose file holds.
fn container(count: usize, payload: usize) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::with_capacity(count * (8 + payload));

  for id in 0..count {
    bytes.extend_from_slice(&(id as u32).to_le_bytes());
    bytes.extend_from_slice(&(payload as u32).to_le_bytes());
    bytes.extend(std::iter::repeat_n(0xAB, payload));
  }

  bytes
}

/// Reads every chunk and its body, which is what a format reader does.
///
/// The payload of each child is consumed rather than skipped: a reader that only walked the headers would report a
/// rate no format achieves, and reading nothing is the failure mode a benchmark hides best.
fn read_all(bytes: &[u8]) -> usize {
  let mut reader: ChunkReader<InMemoryChunkDataSource> =
    ChunkReader::from_bytes(bytes).expect("a well-formed container");

  reader
    .read_children()
    .expect("every child to be well-formed")
    .iter_mut()
    .map(|chunk| chunk.read_remaining().expect("a readable payload").len())
    .sum()
}

fn bench_chunk_reading(criterion: &mut Criterion) {
  let mut group = criterion.benchmark_group("chunk_reading");

  for payload in PAYLOAD_SIZES {
    let bytes: Vec<u8> = container(8, payload);

    group.throughput(Throughput::Bytes(bytes.len() as u64));
    group.bench_with_input(BenchmarkId::new("payload", payload), &bytes, |bencher, bytes| {
      bencher.iter(|| black_box(read_all(black_box(bytes))));
    });
  }

  for count in CHUNK_COUNTS {
    let bytes: Vec<u8> = container(count, SMALL_PAYLOAD);

    group.throughput(Throughput::Elements(count as u64));
    group.bench_with_input(BenchmarkId::new("iteration", count), &bytes, |bencher, bytes| {
      bencher.iter(|| black_box(read_all(black_box(bytes))));
    });
  }

  group.finish();
}

/// Children per parent in the nested shape, matching the writing bench.
const NESTED_CHILDREN: usize = 16;

/// Builds `parents` chunks of [`NESTED_CHILDREN`] small children each.
fn nested_container(parents: usize) -> Vec<u8> {
  let child: Vec<u8> = container(NESTED_CHILDREN, SMALL_PAYLOAD);
  let mut bytes: Vec<u8> = Vec::with_capacity(parents * (8 + child.len()));

  for id in 0..parents {
    bytes.extend_from_slice(&(id as u32).to_le_bytes());
    bytes.extend_from_slice(&(child.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&child);
  }

  bytes
}

/// Reads every parent's children and their bodies, as particles and scene readers walk a two-level tree.
fn read_nested(bytes: &[u8]) -> usize {
  let mut reader: ChunkReader<InMemoryChunkDataSource> =
    ChunkReader::from_bytes(bytes).expect("a well-formed container");

  reader
    .read_children()
    .expect("every parent to be well-formed")
    .iter_mut()
    .map(read_all_children)
    .sum()
}

fn read_all_children<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> usize {
  reader
    .read_children()
    .expect("every child to be well-formed")
    .iter_mut()
    .map(|chunk| chunk.read_remaining().expect("a readable payload").len())
    .sum()
}

/// Builds `count` `[size][content]` records, each size counting its own four bytes.
fn size_packed(count: usize, payload: usize) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::with_capacity(count * (4 + payload));

  for _ in 0..count {
    bytes.extend_from_slice(&((payload + 4) as u32).to_le_bytes());
    bytes.extend(std::iter::repeat_n(0xAB, payload));
  }

  bytes
}

/// The other ways formats walk and pick chunks: nested trees, size-packed records, lookups by id, and a file window.
fn bench_chunk_walking(criterion: &mut Criterion) {
  let mut group = criterion.benchmark_group("chunk_walking");

  for parents in [64, 1024] {
    let bytes: Vec<u8> = nested_container(parents);

    assert_eq!(read_nested(&bytes), parents * NESTED_CHILDREN * SMALL_PAYLOAD);

    group.throughput(Throughput::Elements((parents * NESTED_CHILDREN) as u64));
    group.bench_with_input(BenchmarkId::new("nested", parents), &bytes, |bencher, bytes| {
      bencher.iter(|| black_box(read_nested(black_box(bytes))));
    });
  }

  let count: usize = 1024;
  let mut packed: ChunkReader<InMemoryChunkDataSource> =
    ChunkReader::from_vec(size_packed(count, SMALL_PAYLOAD)).expect("size-packed records");
  let read_packed = |reader: &mut ChunkReader<InMemoryChunkDataSource>| -> usize {
    ChunkSizePackedIterator::from_start(reader)
      .expect("a seekable source")
      .map(|record| {
        record
          .expect("a well-formed record")
          .read_remaining()
          .expect("a readable record")
          .len()
      })
      .sum()
  };

  assert_eq!(read_packed(&mut packed), count * SMALL_PAYLOAD);

  group.throughput(Throughput::Elements(count as u64));
  group.bench_function(BenchmarkId::new("size_packed", count), |bencher| {
    bencher.iter(|| black_box(read_packed(&mut packed)));
  });

  // The last of 64 siblings, the worst case of the linear scan every `find_*_by_id` makes.
  let siblings: Vec<ChunkReader<InMemoryChunkDataSource>> = ChunkReader::from_vec(container(64, SMALL_PAYLOAD))
    .expect("a well-formed container")
    .read_children()
    .expect("every child to be well-formed");

  assert_eq!(
    ChunkReader::find_required_by_id(&siblings, 63)
      .expect("the last sibling")
      .id,
    63
  );

  group.throughput(Throughput::Elements(1));
  group.bench_function(BenchmarkId::new("find_by_id", siblings.len()), |bencher| {
    bencher.iter(|| black_box(ChunkReader::find_required_by_id(black_box(&siblings), black_box(63))));
  });

  // The reader's default source, every header and body read positionally from the file.
  let path: String = GeneratedResources::sample_name(file!(), "container.bin");

  GeneratedResources::write(&path, container(count, SMALL_PAYLOAD)).expect("a written container");

  let mut file_reader: ChunkReader<FileSlice> =
    ChunkReader::from_slice(GeneratedResources::open_slice(&path).expect("the container file")).expect("a file window");

  assert_eq!(read_all_children(&mut file_reader), count * SMALL_PAYLOAD);

  group.throughput(Throughput::Elements(count as u64));
  group.bench_function(BenchmarkId::new("iteration_file", count), |bencher| {
    bencher.iter(|| black_box(read_all_children(&mut file_reader)));
  });

  group.finish();
}

criterion_group!(benches, bench_chunk_reading, bench_chunk_walking);
criterion_main!(benches);
