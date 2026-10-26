# xrf-lzhuf

Compresses and decompresses X-Ray LZHUF payloads: the LZSS plus adaptive Huffman codec behind the engine's
`_compressLZ` and `_decompressLZ`. X-Ray marks a chunk ID with bit 31 to say its payload is coded this way. Archive
file descriptors are the most visible user: [xrf-archive](../xrf-archive/README.md) decodes a volume's descriptor
chunk with this crate and `xrf-pack` encodes it. The engine applies the same coding to any chunk written through
`IWriter::w_chunk`.

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

Both forms return the original bytes. The second form skips the four-byte size header because the caller already sized
the output buffer.

## Blob and stream formats

`LzhufEncoder::compress(&[u8])` returns a blob with a little-endian `u32` decompressed-size header followed by the bit
stream, ready to be written as the payload of a chunk whose ID carries the compressed mark. Pass that complete blob to
`LzhufDecoder::decompress`. Use `LzhufDecoder::decompress_into(stream, target)` only when the stream has no size header
and the output size is already known; it fills the target completely.

| API                             | Input                         | Output                                 |
| ------------------------------- | ----------------------------- | -------------------------------------- |
| `LzhufEncoder::compress`        | Uncompressed bytes            | Four-byte size header and coded stream |
| `LzhufDecoder::decompress`      | Complete blob with its header | Newly allocated bytes                  |
| `LzhufDecoder::decompress_into` | Headerless coded stream       | Caller-supplied output buffer          |

Streams arrive whole in memory, so these APIs take and return slices rather than readers. They do not read files, write
chunk IDs, or manage archive volumes. The high-bit compressed marker belongs to the enclosing chunk header
([xrf-chunk](../xrf-chunk/README.md)), outside this blob. Archive file payloads are LZO-compressed, not LZHUF, and are
not this crate's concern.

## Errors and compatibility

Compression rejects empty input, which no X-Ray decoder accepts, and input too large for the `u32` size header.
`decompress` rejects a blob too short for its header, a declared size of zero, and a declared size larger than the
stream could possibly yield, before allocating; both decoders reject a stream that ends before the output is full or
decodes to a malformed coder state. A successful decode proves that the requested output was produced; it checks no
checksum. Archive entries carry a CRC of their unpacked payload, which `xrf-archive` verifies.

Output is compatible with X-Ray but need not be byte-identical to the engine's: the engine also matches against the
pre-filled history window, which this encoder does not.

The coding is Okumura's LZHUF with one X-Ray deviation, `REBUILD_FREQUENCY_LIMIT`: the adaptive tree is rebuilt at half
the frequency standard LHA `-lh1-` uses. That single constant is why a stock LZH library decodes the first 16384
symbols of an X-Ray stream and then diverges, so generic LZH implementations are not substitutes.

## Checks

Run `cargo test --locked -p xrf-lzhuf` from the repository root. See the [encoder](src/encode/lzhuf_encoder.rs),
[decoder](src/decode/lzhuf_decoder.rs), [constants](src/lzhuf_constants.rs), and [tests](src/tests).
