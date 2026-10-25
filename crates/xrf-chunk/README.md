# xrf-chunk

Reads and writes X-Ray binary chunks, the framing most engine formats share: a `u32` ID and a `u32` payload size
followed by the payload bytes, with children nested inside a parent's payload. Every chunked format crate (OGF, OMF,
THM, level, spawn, material and the rest) reads and writes through this one; it knows no format itself.

## Write and read a chunk

```rust
use xrf_chunk::{ChunkReader, ChunkWriter, XRayByteOrder};

fn main() -> xrf_error::XrfResult {
  let mut writer = ChunkWriter::new();
  writer.buffer.extend_from_slice(b"data");

  let bytes = writer.flush_chunk_into_buffer::<XRayByteOrder>(7)?;

  let mut root = ChunkReader::from_vec(bytes)?;
  let mut child = root.read_child_by_index(0)?;

  assert_eq!(child.id, 7);
  assert_eq!(child.read_remaining()?, b"data");

  Ok(())
}
```

The outer reader spans the whole buffer. The child's ID is `7`, while its position in the child sequence is `0`:
`read_child_by_index` selects an ordinal, not a stored ID.

## Sources and readers

- Open owned bytes with `ChunkReader::from_vec`, bytes the caller keeps reading as a slice with `from_shared` (an
  `Arc<Vec<u8>>`, so several passes share one buffer), copied bytes with `from_bytes`, an `InMemoryChunkDataSource` with
  `from_source`, a file window with `from_slice`, or a path with `from_path`, which reads positionally rather than
  holding the file. `from_file` loads the whole file into memory. Every constructor rejects an empty source; an empty
  child chunk is ordinary.
- Locate children with `read_children` or `ChunkIterator`, then pick one with `ChunkReader::find_required_by_id`,
  `find_optional_by_id` or their `find_one_of_*` forms, which match the stored ID exactly and take a preference order
  of IDs. `ChunkSizePackedIterator` walks a payload stored as `[size][content]` records, where each size counts its own
  four bytes.
- `read_u8_payload`, `read_u32_payload`, `read_w1251_string_payload` and their kin read a chunk whose whole payload is
  one value, and fail when bytes remain after it.
- Build a payload with `ChunkWriter`, then use `flush_chunk_into` or `flush_chunk_into_buffer` for a framed chunk, or
  `flush_raw_into` for raw bytes.
- Implement `ChunkReadWrite` for a format's typed payload; `ChunkReadWriteList` and `ChunkReadWriteOptional` cover
  repeated or absent chunks, and `read_xr`, `write_xr` and their list and optional forms delegate to them.

Readers track their current position and remaining byte budget. `read_children` consumes the child sequence from the
start; `get_children_cloned` reads through a clone without moving the original cursor. Each child is a window cut from
its parent's source: in-memory windows share one buffer, so slicing and cloning do not copy, and an in-memory child's
`position` and cursor count from its parent window's start rather than the file's.

## Framing and typed payloads

A chunk header is two `u32` fields, ID then payload size (`CHUNK_HEADER_SIZE`). `ChunkWriter` buffers the payload; its
flush methods read that buffer without changing it, so clear it or create another writer before constructing an
unrelated payload. A payload larger than a `u32` can declare is refused.

`XRayByteOrder` is little-endian. Payload reads and the writer's header take the byte order as a type parameter, but
the child iterators always read headers little-endian.

Strings are null-terminated Windows-1251, read up to 10,240 bytes unless a `_limited` form gives another bound.
`read_w1251_line` reads a value ended by any run of `\r` and `\n`, and returns a `ChunkLine` carrying that terminator so
`write_w1251_line` can put back exactly what the file held. `read_f32_q16` and `write_f32_q16` read and write a float
quantised into a `u16` over a range the format supplies; a write clamps a value outside the range, and only the bytes,
not the float, survive a round trip. `new_bounded_vec` reserves for a declared record count only after checking it
against the bytes remaining.

## Errors, trailing bytes and compressed chunks

Empty sources, missing required children, incomplete headers, sizes reaching past the source, and reads outside the
remaining budget return errors. `assert_read` fails when a reader has bytes left; format readers call it to prove they
consumed a whole payload.

`read_children` fails on the first malformed header. `read_children_with_trailing` returns the well-formed children
and a `ChunkTrailing` with the unaccounted bytes and the error a strict walk would raise, leaving the decision to the
format.

A chunk whose ID carries `CHUNK_ID_COMPRESSED_MASK` (bit 31) holds a compressed payload. Ordinary child reads reject
it. `read_children_including_compressed` returns the marked ID and the stored bytes without decoding them; the caller
must interpret the compressed payload explicitly. X-Ray's codec for it is [xrf-lzhuf](../xrf-lzhuf/README.md).

## Checks

Run `cargo test --locked -p xrf-chunk` from the repository root. Typed format tests belong to the crate implementing the
payload, while this crate checks framing, boundaries, sources, and primitive I/O.

See the [reader](src/reader/chunk_reader.rs), [child iterator](src/iterator/chunk_iterator.rs),
[writer](src/writer/chunk_writer.rs), [in-memory source](src/source/chunk_memory_source.rs), and [exports](src/lib.rs).
