# xrf-efd

Reads, writes and evaluates `.efd` evaluation function data, the pattern tables the game's AI scores choices with.

Each file is one trained evaluation function, the engine's `CPatternFunction` (`xrGame/ef_pattern.cpp`): a table of
weights looked up by the bucketed values of a few input functions. `EfdFile` holds it; the desktop app's archive
describer reads it.

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

- Read with `read_from_path`, `read_from_file`, `read_from_bytes` (for an archived entry), or `read` for a reader at
  the file's first byte. `XRayByteOrder` selects the little-endian layout.
- `write` produces the complete file in the writer's buffer; there is no chunk header to add.

## Layout

The file is flat, not a chunk tree. Nothing can be skipped, so a read either accounts for every byte or fails:

1. A `u32` builder version, which must be `CURRENT_VERSION` (1), and a `u32` data format.
2. A `u32` variable count, then the variables' ranges and then their kinds as two parallel arrays. `EfdVariable` joins
   them: `range` is how many discrete values the input is bucketed into, `kind` the function type of the function
   supplying it.
3. The `u32` function type this one registers as, then the minimum and maximum results.
4. A `u32` pattern count, and per pattern a `u32` cardinality and that many variable indices. `EfdPattern::variables`
   indexes the function's own variable list.
5. The weights, laid out pattern by pattern.

The data format sets the width of results and weights:

| Format                      | Stored as | Notes                                                    |
| --------------------------- | --------- | -------------------------------------------------------- |
| `EfdDataFormat::Float` (1)  | `f32`     | Every shipped function; the only layout the engine reads |
| `EfdDataFormat::Double` (0) | `f64`     | What the SDK's builder writes unless asked to convert    |

The engine's `vfLoadEF` never checks the field, so `is_engine_readable` says whether a function will load as what it
is. `EfdFile` keeps every real as `f64`, so a float file widens exactly and writes back byte for byte.

The weight count is not stored. It is the sum, over patterns, of the product of their variables' ranges
(`EfdPattern::get_complexity` for one pattern), so the reader walks every pattern before sizing the weight block.

## Evaluate a function

`evaluate` takes one bucketed value per variable and returns what `CPatternFunction::ffEvaluate` does:

1. Each pattern reads its inputs' values as one mixed-radix number, first input most significant
   (`EfdPattern::get_index`, `dwfGetPatternIndex`).
2. That number selects a weight from the pattern's own stretch of the block.
3. The selected weights are summed in pattern order: in `f32` for a float function, as the engine does, and in `f64`
   for a double one, as the builder does.

In the game, the inputs come from other functions. A variable's `kind` names the `CEF_Storage` slot that supplies it:
either a primary function computed from the game state, or another pattern function's `function_type`.

A pattern function passes its result on through `get_discrete_value(result, range)`
(`CBaseFunction::dwfGetDiscreteValue`): its minimum and maximum are split into `range` evenly spaced buckets, and the
result rounds to the nearest.

The crate evaluates one function at a time. It does not compute primary functions, which need a running game.

## Errors

Reads reject:

- a file that cannot be opened;
- another builder version, or a data format the builder never writes;
- declared counts the remaining bytes cannot hold;
- a pattern naming a variable past the list;
- a weight block shorter than the patterns claim;
- trailing bytes.

The engine reads past a pattern's list into memory it does not own and ignores bytes after the weights; both are
refused here.

Writes reject a weight count that disagrees with the patterns, and counts too large for a `u32`. Every other field is
written as given.

Evaluation rejects a value list of the wrong length, or a value past its variable's range.

## Checks

Run `cargo test --locked -p xrf-efd` from the repository root. See the [function](src/efd_file.rs),
[data format](src/efd_data_format.rs), [pattern](src/efd_pattern.rs), [variable](src/efd_variable.rs), and
[format tests](src/tests.rs).
