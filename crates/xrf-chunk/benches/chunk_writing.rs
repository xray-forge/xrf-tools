//! What framing chunks costs on the way out: flat containers, nested trees and payload sizes.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use xrf_chunk::{CHUNK_HEADER_SIZE, ChunkReader, ChunkWriter};

/// Payload sizes for the bytes-dominated shape, matching the reading bench.
const PAYLOAD_SIZES: [usize; 3] = [4 * 1024, 256 * 1024, 4 * 1024 * 1024];

/// Chunk counts for the overhead-dominated shape, matching the reading bench.
const CHUNK_COUNTS: [usize; 3] = [64, 1024, 16384];

/// Bytes per chunk in the overhead-dominated shape.
const SMALL_PAYLOAD: usize = 16;

/// Children per parent in the nested shape, near a particle group's effect list.
const NESTED_CHILDREN: usize = 16;

/// Writes `count` chunks of `payload` bytes into one container, each built in its own writer and flushed into the
/// parent, which is how format writers assemble a file.
fn write_flat(count: usize, payload: &[u8]) -> Vec<u8> {
  let mut parent: ChunkWriter = ChunkWriter::new();

  for id in 0..count {
    let mut child: ChunkWriter = ChunkWriter::new();

    child.buffer.extend_from_slice(payload);
    child
      .flush_chunk_into(&mut parent.buffer, id as u32)
      .expect("a framed child");
  }

  parent.flush_chunk_into_buffer(0).expect("a framed container")
}

/// Writes `parents` chunks of [`NESTED_CHILDREN`] small children each, so every byte is copied once per level.
fn write_nested(parents: usize) -> Vec<u8> {
  let payload: [u8; SMALL_PAYLOAD] = [0xAB; SMALL_PAYLOAD];
  let mut root: ChunkWriter = ChunkWriter::new();

  for parent_id in 0..parents {
    let mut parent: ChunkWriter = ChunkWriter::new();

    for child_id in 0..NESTED_CHILDREN {
      let mut child: ChunkWriter = ChunkWriter::new();

      child.buffer.extend_from_slice(&payload);
      child
        .flush_chunk_into(&mut parent.buffer, child_id as u32)
        .expect("a framed child");
    }

    parent
      .flush_chunk_into(&mut root.buffer, parent_id as u32)
      .expect("a framed parent");
  }

  root.flush_chunk_into_buffer(0).expect("a framed root")
}

fn bench_chunk_writing(criterion: &mut Criterion) {
  let mut group = criterion.benchmark_group("chunk_writing");

  for size in PAYLOAD_SIZES {
    let payload: Vec<u8> = vec![0xAB; size];
    let written: Vec<u8> = write_flat(8, &payload);

    assert_eq!(written.len() as u64, CHUNK_HEADER_SIZE * 9 + size as u64 * 8);
    assert_eq!(
      ChunkReader::from_vec(written)
        .expect("the written container")
        .read_child_by_index(0)
        .expect("its root")
        .read_children()
        .expect("its children")
        .len(),
      8
    );

    group.throughput(Throughput::Bytes((size * 8) as u64));
    group.bench_with_input(BenchmarkId::new("payload", size), &payload, |bencher, payload| {
      bencher.iter(|| black_box(write_flat(8, black_box(payload))));
    });
  }

  let payload: [u8; SMALL_PAYLOAD] = [0xAB; SMALL_PAYLOAD];

  for count in CHUNK_COUNTS {
    group.throughput(Throughput::Elements(count as u64));
    group.bench_with_input(BenchmarkId::new("framing", count), &count, |bencher, count| {
      bencher.iter(|| black_box(write_flat(*count, black_box(&payload))));
    });
  }

  for parents in [64, 1024] {
    let written: Vec<u8> = write_nested(parents);

    assert_eq!(
      written.len() as u64,
      CHUNK_HEADER_SIZE * (1 + parents as u64 * (1 + NESTED_CHILDREN as u64))
        + (parents * NESTED_CHILDREN * SMALL_PAYLOAD) as u64
    );

    group.throughput(Throughput::Elements((parents * NESTED_CHILDREN) as u64));
    group.bench_with_input(BenchmarkId::new("nested", parents), &parents, |bencher, parents| {
      bencher.iter(|| black_box(write_nested(*parents)));
    });
  }

  group.finish();
}

criterion_group!(benches, bench_chunk_writing);
criterion_main!(benches);
