# xrf-chunk

Reads and writes X-Ray binary chunks: an ID and payload size followed by the payload bytes.

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

The outer reader spans the whole buffer. The child's ID is `7`, while its position in the child sequence is `0`.
`XRayByteOrder` is little-endian; formats with a different byte order must choose it explicitly.

## Sources and readers

- Open owned bytes with `ChunkReader::from_vec`, copied bytes with `from_bytes`, or a file window with `from_slice`.
  `from_file` loads the whole file into memory. Empty sources are rejected.
- Use `ChunkIterator` and the `find_*_chunk_by_id` helpers to locate children.
- Build a payload with `ChunkWriter`, then use `flush_chunk_into` for a framed chunk or `flush_raw_into` for raw bytes.
- Implement `ChunkReadWrite` for a format's typed payload; the list and optional traits cover repeated or absent chunks.

Readers track their current position and remaining byte budget. `read_children` consumes the child sequence;
`get_children_cloned` reads through a clone without moving the original cursor. The returned children retain views
of the source rather than exposing unrelated file offsets to the caller.

## Framing and typed payloads

A chunk header is two `u32` fields: ID and payload size. `ChunkWriter` buffers the payload; its flush methods leave
that buffer intact. Clear it or create another writer before constructing an unrelated payload.

The `ChunkReadWrite` traits keep typed format logic separate from transport. `read_xr`, `write_xr`, and their list or
optional forms delegate to those traits. String helpers read and write the Windows-1251 forms used by engine formats.

## Errors and compressed chunks

Empty sources, missing required children, invalid sizes, and reads outside the remaining budget return errors.
`read_child_by_index` selects an ordinal, not a stored ID; use the `find_*_chunk_by_id` helpers for ID lookup.

Ordinary child reads reject compressed-marked chunks. `read_children_including_compressed` exposes their stored bytes
and marked IDs without decoding them; the caller must interpret the compressed payload explicitly.

Choose the byte order required by the format. Chunk framing does not identify the payload's schema; those readers live
in [xrf-db](../xrf-db/README.md). X-Ray's compressed payload codec is [xrf-lzhuf](../xrf-lzhuf/README.md).

## Checks

Run `cargo test --locked -p xrf-chunk` from the repository root. Typed format tests belong to the crate implementing
the payload, while this crate checks framing, boundaries, sources, and primitive I/O.

See the [reader](src/reader/chunk_reader.rs), [writer](src/writer/chunk_writer.rs), and [exports](src/lib.rs).
