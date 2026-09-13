# xrf-output

Routes live workflow messages to a caller-selected sink with shared channels and verbosity rules.

## Capture workflow messages

```rust
use std::sync::Arc;
use xrf_output::{OutputOptions, OutputVerbosity, RecordingOutput};

fn main() {
  let sink = Arc::new(RecordingOutput::default());
  let output = OutputOptions::new(sink.clone(), OutputVerbosity::Normal);

  xrf_output::info!(output, "Read {} files", 3);
  xrf_output::verbose!(output, "This detail is hidden at normal verbosity");
  output.warning("One optional asset is missing");

  assert_eq!(sink.list_records().len(), 2);
}
```

Implement `Output` for a terminal, UI, or another destination. `RecordingOutput` retains records for inspection;
`NoopOutput` discards them.

## Channels and verbosity

Pass `OutputOptions` to the operation. Emit messages through methods such as `info` and `warning`, or the corresponding
formatting macros. The options filter channels before handing messages to the sink.

| Verbosity | Delivered channels                              |
| --------- | ----------------------------------------------- |
| `Silent`  | Error, failure                                  |
| `Normal`  | Error, failure, heading, success, warning, info |
| `Verbose` | All channels, including verbose detail          |

Default options use `NoopOutput`. Check `is_visible` before building an expensive message; visibility alone does not
mean that the selected sink displays or retains it.

## Preserve worker order

```rust
use xrf_output::{OutputOptions, OutputSequence};

fn main() {
  let output = OutputOptions::default();
  let sequence = OutputSequence::new(&output, 2);
  let second = sequence.new_slot(1);
  let first = sequence.new_slot(0);

  second.get_output().info("second");
  drop(second);

  first.get_output().info("first");
  drop(first);
}
```

Each slot publishes when dropped, after earlier slots have finished. Create the slot before fallible work so an early
return still releases its position. Give each work item its own input-order index; output ordering does not schedule
or limit the workers.

## Checks and related crates

Progress and cancellation belong to [xrf-job](../xrf-job/README.md); finalized findings belong to
[xrf-report](../xrf-report/README.md).

Run `cargo test --locked -p xrf-output`. An output sink handles messages, not operation results: emitting a failure
message does not itself return an error or mark a report as failed.

See the [options and tests](src/options.rs), [sinks](src/output.rs), and [ordered output](src/sequence.rs).
