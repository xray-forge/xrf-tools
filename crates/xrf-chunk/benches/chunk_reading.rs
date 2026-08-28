//! What the chunk reader costs per byte and per chunk.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use xrf_chunk::{ChunkReader, InMemoryChunkDataSource};

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

criterion_group!(benches, bench_chunk_reading);
criterion_main!(benches);
