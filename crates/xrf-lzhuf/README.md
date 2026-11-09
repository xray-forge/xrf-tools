# xrf-lzhuf

Compresses and decompresses X-Ray LZHUF payloads: the LZSS plus adaptive Huffman codec behind the engine's
`_compressLZ` and `_decompressLZ`.

X-Ray marks a chunk ID with bit 31 when its payload is coded this way. The engine can code any chunk written through
`IWriter::w_chunk`; archive file descriptors are the most visible case: [xrf-archive](../xrf-archive/README.md) decodes
a volume's descriptor chunk with this crate, and `xrf-pack` encodes it.

## Compress and read back

```rust
use xrf_lzhuf::{LzhufDecoder, LzhufEncoder};

fn main() -> xrf_error::XrfResult {
  let source = b"configs\\system.ltx configs\\system.ltx";
  let packed = LzhufEncoder::compress(source)?;

  assert_eq!(LzhufDecoder::decompress(&packed)?, source);

  let mut output = vec![0; source.len()];

  LzhufDecoder::decompress_into(&packed[4..], &mut output)?;
  assert_eq!(output, source);

  Ok(())
}
```

## Blobs and streams

| API                             | Input                         | Output                                 |
| ------------------------------- | ----------------------------- | -------------------------------------- |
| `LzhufEncoder::compress`        | Uncompressed bytes            | Four-byte size header and coded stream |
| `LzhufDecoder::decompress`      | Complete blob with its header | Newly allocated bytes                  |
| `LzhufDecoder::decompress_into` | Headerless coded stream       | Caller-supplied output buffer          |

- The size header is a little-endian `u32` of the decompressed size. A compressed blob is ready to be the payload of a
  chunk whose ID carries the compressed mark.
- Use `decompress_into` only when the stream has no header and the output size is already known; it fills the target
  completely.
- Everything works on in-memory slices. The crate does not read files, write chunk IDs, or manage archive volumes; the
  compressed mark belongs to the chunk header ([xrf-chunk](../xrf-chunk/README.md)).
- Archive file payloads are LZO-compressed, not LZHUF, and are not handled here.

## Errors

Compression rejects:

- empty input, which no X-Ray decoder accepts;
- input too large for the `u32` size header.

`decompress` rejects, before allocating, a blob too short for its header, a declared size of zero, and a declared size
larger than the stream could yield. Both decoders reject a stream that ends before the output is full or reaches a
malformed coder state.

A successful decode only proves the output was produced; there is no checksum. Archive entries carry a CRC of their
unpacked payload, which `xrf-archive` checks.

## Compatibility

- Output decodes in X-Ray, but need not match the engine's bytes: the engine also matches against the pre-filled
  history window, which this encoder does not.
- The coding is Okumura's LZHUF with one X-Ray change, `REBUILD_FREQUENCY_LIMIT`: the adaptive tree is rebuilt at half
  the frequency of standard LHA `-lh1-`. A stock LZH library therefore decodes the first 16384 symbols of an X-Ray
  stream and then diverges; generic LZH implementations are not substitutes.

## Checks

Run `cargo test --locked -p xrf-lzhuf` from the repository root.

Sources: the [encoder](src/encode/lzhuf_encoder.rs), [decoder](src/decode/lzhuf_decoder.rs),
[constants](src/lzhuf_constants.rs), and [tests](src/tests).
