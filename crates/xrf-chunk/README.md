# xrf-chunk

Reads and writes X-Ray binary chunks, the framing most engine formats share: a `u32` ID, a `u32` payload size, then the
payload bytes. Children nest inside a parent's payload.

Every chunked format crate (OGF, OMF, THM, level, spawn, material and the rest) reads and writes through this one. It
knows no format itself.

## Write and read a chunk

```rust
use xrf_chunk::{ChunkReader, ChunkWriter};

fn main() -> xrf_error::XrfResult {
  let mut writer = ChunkWriter::new();
  writer.buffer.extend_from_slice(b"data");

  let bytes = writer.flush_chunk_into_buffer(7)?;

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

Open a reader from:

| Constructor   | Source                                                 |
| ------------- | ------------------------------------------------------ |
| `from_vec`    | Owned bytes                                            |
| `from_shared` | An `Arc<Vec<u8>>`, so several passes share one buffer  |
| `from_bytes`  | A slice, copied                                        |
| `from_source` | An `InMemoryChunkDataSource`                           |
| `from_slice`  | A window of a file                                     |
| `from_path`   | A path, read positionally rather than holding the file |
| `from_file`   | A file, loaded whole into memory                       |

Every constructor rejects an empty source; an empty child chunk is ordinary.

Then:

- Locate children with `read_children` or `ChunkIterator`, then pick one with `ChunkReader::find_required_by_id`,
  `find_optional_by_id` or their `find_one_of_*` forms, which match the stored ID exactly and take a preference order
  of IDs. `ChunkSizePackedIterator` walks a payload stored as `[size][content]` records, where each size counts its own
  four bytes.
- `read_u8_payload`, `read_u32_payload`, `read_w1251_string_payload` and their kin read a chunk whose whole payload is
  one value, and fail when bytes remain after it.
- To decode a run of bytes without keeping them, use `read_remaining_with`, `read_with`, or a source's `read_at_with`:
  an in-memory source lends the bytes where they lie, any other reads them into a buffer first. OGF vertices and level
  render geometry decode this way. A `read_at_with` range ending past `u64` is an `InvalidInput` error.
- An array of fixed-size records reads with `read_records` (a count) or `read_remaining_records` (the rest of the
  chunk) and writes with `write_records`, each in one pass over the bytes. The record type implements
  `ChunkRecord<SIZE>`, whose `decode` and `encode` state its layout once; `u16`, `u32`, `f32` and `xrf-math`'s
  `Vector3d<f32>` already do. Reading such an array one field at a time is about nine times slower.
- Build a payload with `ChunkWriter` and place it in its parent framed with `flush_chunk_into(&mut parent, id)`, or raw
  with `flush_raw_into`; bytes already held frame with `ChunkWriter::write_framed`, never copied into a writer first.
  Take a root's bytes with `into_bytes`. `flush_chunk_into_buffer` gives a framed chunk as a value, for a caller that
  wants one; it is not how a child joins its parent.
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

Every X-Ray format is little-endian, and so is everything here. `XRayRead` and `XRayWrite` read and write each number
width on any reader or writer, `ChunkReader` and `ChunkWriter` among them; `XRayBytes` does the same at the front of a
byte slice already in hand. No other crate names a byte order.

Strings are null-terminated Windows-1251, read up to 10,240 bytes unless a `_limited` form gives another bound, and
decoded through `xrf_utils::decode_w1251_field_to_string`. No byte order mark is looked for: a string opening on
`EF BB BF` reads as `п»ї` and writes back as the same bytes.

`read_w1251_line` reads a value ended by any run of `\r` and `\n`, and returns a `ChunkLine` carrying that terminator
so `write_w1251_line` can put back exactly what the file held. `read_f32_q16` and `write_f32_q16` read and write a float
quantised into a `u16` over a range the format supplies; a write clamps a value outside the range, and only the bytes,
not the float, survive a round trip. `new_bounded_vec` reserves for a declared record count only after checking it
against the bytes remaining.

## Values as bytes and as fields

The engine's `NET_Packet` writes each value through a primitive (`w_u16`, `w_float`, `w_vec3`, `w_stringZ`). With an
ini stream attached it also saves one field per primitive, which is how the Level Editor keeps an entity's spawn data.
`ChunkFieldWriter` and `ChunkFieldReader` are those primitives: `u8`, `u16`, `u32`, `u64`, `f32`, a three-float vector,
a Windows-1251 string, raw bytes, the position (`w_tell`), a `u16` patch (`w_seek`), the end (`r_eof`), and
`is_field_form` (`inistream != nullptr`) for a value the engine writes differently to an ini stream. Provided methods
(`read_xr`, `read_xr_list`, `read_xr_optional`, `read_u16_vector`, `new_bounded_vec` and their writer twins) carry
`ChunkReader`'s and `ChunkWriter`'s names, so a value's code reads the same over either form.

- `ChunkWriter` and `ChunkReader` implement the traits as bytes.
- `ChunkFieldRecorder` records each value as a `ChunkField` of its `ChunkFieldKind`; `ChunkFieldReplay` reads them back
  from any `ChunkFieldSource`. Both count the bytes the values would take, so a size or marker taken from their position
  matches the byte form. A patch leaves recorded fields as written; raw bytes make no field and are refused unless
  empty; `new_bounded_vec` checks a count against the bytes left only in the byte form.
- A type implementing `ChunkFieldReadWrite` (or its `List` and `Optional` forms) gets `ChunkReadWrite` (or the matching
  form) from a blanket impl, so its bytes are its fields over a `ChunkWriter` and the two forms cannot drift. Every
  `xrf-math` chunk type and every `xrf-spawn` ALife class and value is written this way.

The traits' methods share their names with `XRayRead`'s and `XRayWrite`'s (`write_u16`), so a file using both on a
`ChunkWriter` calls one through its trait.

## Layout of a container

`ChunkLayout` lets a format that finds chunks by id rewrite a file byte for byte. It keeps the children's stored order
and, as `ChunkPayload`s (id and bytes), every child no field takes.
[xrf-editable-object](../xrf-editable-object/README.md) keeps one per container.

- `ChunkLayout::read` hands each child to a closure that folds it into a field and answers `true`, or answers `false`
  to keep it.
- `ChunkLayout::write` takes the format's children encoded into a `ChunkSequence` and writes them in the stored order,
  each id taking the first unwritten child with that id, then the rest, kept children last.
- A default layout keeps nothing and writes the format's own order.

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

Sources: the [reader](src/reader/chunk_reader.rs), [child iterator](src/iterator/chunk_iterator.rs),
[writer](src/writer/chunk_writer.rs), [in-memory source](src/source/chunk_memory_source.rs),
[field read-write](src/read_write/chunk_field_read_write.rs), and [exports](src/lib.rs).
