# xrf-efd

Reads, writes and evaluates `.efd` evaluation function data, the pattern tables the game's AI evaluates. Each file is
one trained evaluation function, the engine's `CPatternFunction` (`xrGame/ef_pattern.cpp`): a weighted table looked up
by the bucketed values of a few input functions. `EfdFile` holds it; the desktop app's archive describer reads it.

## Read, evaluate and write a function

```rust
use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_efd::{EfdDataFormat, EfdFile, EfdPattern, EfdVariable};

fn main() -> xrf_error::XrfResult {
  let function = EfdFile {
    builder_version: EfdFile::CURRENT_VERSION,
    data_format: EfdDataFormat::Float,
    function_type: 76,
    minimum_result: 0.0,
    maximum_result: 100.0,
    variables: vec![EfdVariable { range: 3, kind: 10 }, EfdVariable { range: 5, kind: 11 }],
    patterns: vec![EfdPattern { variables: vec![0, 1] }],
    // One weight per combination of the pattern's inputs: 3 * 5.
    parameters: (0..15).map(f64::from).collect(),
  };

  assert_eq!(function.patterns[0].get_complexity(&function.variables), Some(15));
  // Input 0 at bucket 2 and input 1 at bucket 4 select weight 2 * 5 + 4.
  assert_eq!(function.evaluate(&[2, 4])?, 14.0);

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
   them: `range` is how many discrete values the input is bucketed into, `kind` the function type of the function
   supplying it.
3. The `u32` function type this one registers as, then the minimum and maximum results.
4. A `u32` pattern count, and per pattern a `u32` cardinality and that many variable indices. `EfdPattern::variables`
   indexes the function's own variable list.
5. The weights, laid out pattern by pattern.

The data format decides how wide the results and weights are. `EfdDataFormat::Float` (1) stores them as `f32`, the
layout every shipped function uses; `EfdDataFormat::Double` (0) stores them as `f64`, which the SDK's builder writes
unless it is asked to convert. The engine reads only the float layout: `vfLoadEF` never looks at the field, so
`is_engine_readable` says whether a function will load as what it is. `EfdFile` keeps every real as `f64` whatever the
format, so a float file widens exactly and is written back byte for byte.

The weight count is never stored. It is the sum over the patterns of the product of their variables' ranges, which
`EfdPattern::get_complexity` gives for one pattern, so the reader walks every pattern before it can size the block.

## Evaluate a function

`evaluate` takes one bucketed value per variable and answers what `CPatternFunction::ffEvaluate` does: each pattern
reads its inputs' values as one mixed-radix number, its first input the most significant (`EfdPattern::get_index`,
`dwfGetPatternIndex`), selects that weight from its own stretch of the block, and the selected weights are summed in
pattern order. A float function sums in `f32` as the engine does, and a double one in `f64` as the builder does.

The engine fills those values from other functions. A variable's `kind` names the slot of `CEF_Storage` that supplies
it, which is either a primary function computed from the game or another pattern function's `function_type`. A pattern
function hands its result over through `get_discrete_value(result, range)`, `CBaseFunction::dwfGetDiscreteValue`: its
own minimum and maximum spread `range` evenly spaced buckets, the reading variable's range, and a result rounds to the
nearest. The crate evaluates one function at a time and does not compute primary functions, which need a running game.

## Errors and checks

Reads reject a file that cannot be opened, another builder version, a data format the builder never writes, declared
counts the remaining bytes cannot hold, a pattern naming a variable past the list, a weight block shorter than the
patterns claim, and trailing bytes. The engine reads past a pattern's list into memory it does not own and ignores
bytes after the weights; both are refused here instead. Writes reject a weight count that disagrees with the patterns
and counts too large for a `u32`, and keep every other field as given. Evaluation rejects a value list of the wrong
length or a value past its variable's range.

Run `cargo test --locked -p xrf-efd` from the repository root. See the [function](src/efd_file.rs),
[data format](src/efd_data_format.rs), [pattern](src/efd_pattern.rs), [variable](src/efd_variable.rs), and
[format tests](src/tests.rs).
