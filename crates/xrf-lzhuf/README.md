# xrf-lzhuf

Compresses and decompresses X-Ray LZHUF payloads: LZSS with adaptive Huffman coding, as used by compressed chunks.

## Compress and read back

```rust
use xrf_lzhuf::{compress, decompress, decompress_into};

fn main() -> xrf_error::XrfResult {
  let source = b"configs\\system.ltx configs\\system.ltx";
  let packed = compress(source)?;

  assert_eq!(decompress(&packed)?, source);

  let mut output = vec![0; source.len()];

  decompress_into(&packed[4..], &mut output)?;
  assert_eq!(output, source);

  Ok(())
}
```

Both forms return the original bytes. The second form removes the size header because the caller already sized the
output buffer.

## Blob and stream formats

`compress(&[u8])` returns a blob with a little-endian `u32` decompressed-size header. Pass that complete blob to
`decompress`. Use `decompress_into(stream, target)` only when the stream has no size header and the output size is
already known; it fills the target completely.

| API               | Input                         | Output                                 |
| ----------------- | ----------------------------- | -------------------------------------- |
| `compress`        | Uncompressed bytes            | Four-byte size header and coded stream |
| `decompress`      | Complete blob with its header | Newly allocated bytes                  |
| `decompress_into` | Headerless coded stream       | Caller-supplied output buffer          |

These APIs operate on complete in-memory slices. They do not read files, write chunk IDs, or manage archive volumes.
The high-bit compressed marker belongs to the enclosing X-Ray chunk header, outside this blob.

## Errors and compatibility

Compression rejects empty input and input too large for the size header. Decompression rejects malformed or truncated
streams and invalid declared sizes. Output is compatible with X-Ray but need not be byte-identical to the engine's
compressed output.

The decoder bounds a declared output size before allocating. A successful decode proves that the requested output was
produced; it is not an archive CRC check. Use the archive reader when descriptor integrity must also be checked.

X-Ray uses a different Huffman rebuild threshold from standard LZHUF; generic LZH implementations are not substitutes.

## Checks and related code

Run `cargo test --locked -p xrf-lzhuf`. See the [encoder](src/encode/lzhuf_encode.rs),
[decoder](src/decode/lzhuf_decode.rs), and [tests](src/tests). Chunk framing is in
[xrf-chunk](../xrf-chunk/README.md); volume reads are in [xrf-archive](../xrf-archive/README.md).
