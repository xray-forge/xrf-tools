# xrf-efd

Reads and writes `.efd` evaluation function data, the pattern tables the game evaluates. Each file is one trained
evaluation function, the engine's `CPatternFunction` (`xrGame/ef_pattern.cpp`): a weighted table the AI looks up by the
bucketed values of a few input functions. `EfdFile` holds it; the desktop app's archive describer reads it.

## Read and write a function

```rust
use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_efd::{EfdFile, EfdPattern, EfdVariable};

fn main() -> xrf_error::XrfResult {
  let function = EfdFile {
    builder_version: EfdFile::CURRENT_VERSION,
    data_format: 1,
    function_type: 76,
    minimum_result: 0.0,
    maximum_result: 100.0,
    variables: vec![EfdVariable { range: 3, kind: 10 }, EfdVariable { range: 5, kind: 11 }],
    patterns: vec![EfdPattern { variables: vec![0, 1] }],
    // One weight per combination of the pattern's inputs: 3 * 5.
    parameters: vec![0.0; 15],
  };

  assert_eq!(function.patterns[0].get_complexity(&function.variables), Some(15));

  let mut writer = ChunkWriter::new();
  function.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(EfdFile::read_from_bytes::<XRayByteOrder>(writer.buffer)?, function);

  Ok(())
}
```

Read a file with `read_from_path`, `read_from_file`, `read_from_bytes` for an archived entry, or `read` for a reader
positioned at the file's first byte. `XRayByteOrder` selects the little-endian layout. `write` produces the complete
file in the writer's buffer; there is no chunk header to add.

## Layout

The file is flat, not a chunk tree, so nothing is skippable and a read either accounts for every byte or fails:

1. A `u32` builder version, which must be `CURRENT_VERSION` (1), and a `u32` data format.
2. A `u32` variable count, then the variables' ranges and then their kinds as two parallel arrays. `EfdVariable` joins
   them: `range` is how many discrete values the input is bucketed into, `kind` the function type of the base function
   supplying it.
3. The `u32` function type this one registers as, then `f32` minimum and maximum results.
4. A `u32` pattern count, and per pattern a `u32` cardinality and that many variable indices. `EfdPattern::variables`
   indexes the function's own variable list.
5. The `f32` weights, laid out pattern by pattern.

The weight count is never stored. It is the sum over the patterns of the product of their variables' ranges, which
`EfdPattern::get_complexity` gives for one pattern, so the reader walks every pattern before it can size the block.

## Errors and checks

Reads reject a file that cannot be opened, another builder version, declared counts the remaining bytes cannot hold, a
pattern naming a variable past the list, a weight block shorter than the patterns claim, and trailing bytes. Writes
reject a weight count that disagrees with the patterns and counts too large for a `u32`, and keep every other field as
given, so a function read and written back is byte for byte the file it came from. The crate does not evaluate a
function.

Run `cargo test --locked -p xrf-efd` from the repository root. See the [function](src/efd_file.rs),
[pattern](src/efd_pattern.rs), [variable](src/efd_variable.rs), and [format tests](src/tests.rs).
