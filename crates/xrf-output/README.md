# xrf-output

Caller-controlled live output for XRF workflows: the messages a long operation says while it runs, routed to a sink the
caller chooses and filtered by one verbosity rule. An operation takes `OutputOptions` (usually in its options struct)
and never prints; `xrf-cli` renders the messages on the terminal, and tests record them.

## Send messages

```rust
use std::sync::Arc;

use xrf_output::{OutputOptions, OutputVerbosity, RecordingOutput};

let sink: Arc<RecordingOutput> = Arc::new(RecordingOutput::default());
let output: OutputOptions = OutputOptions::new(sink.clone(), OutputVerbosity::Normal);

xrf_output::info!(output, "Read {} files", 3);
xrf_output::verbose!(output, "This detail is hidden at normal verbosity");
output.warning("One optional asset is missing");

assert_eq!(sink.list_records().len(), 2);
```

`OutputOptions` has one method per `OutputChannel` (`heading`, `success`, `warning`, `failure`, `info`, `error`,
`verbose`), each taking any `Display`, and a matching macro (`xrf_output::info!(output, "...", args)`) that formats
without allocating a `String` first. Options clone cheaply and share the sink.

## Channels and verbosity

`OutputOptions` filters by channel before the sink sees a message; a sink never applies verbosity itself.

| Verbosity          | Delivered channels                              |
| ------------------ | ----------------------------------------------- |
| `Silent` (default) | Error, failure                                  |
| `Normal`           | Error, failure, heading, success, warning, info |
| `Verbose`          | All channels, including verbose detail          |

Errors and failures are delivered at every verbosity, so a failing run is never silent. Check
`is_visible(channel)` before building an expensive message. `OutputOptions::default()` is silent and writes to
`NoopOutput`; `with_output` keeps the verbosity and swaps the sink.

## Sinks

Implement the `Output` trait (`Send + Sync`, one `write(channel, message)` method) for a terminal, UI, or another
destination; the CLI's terminal sink is one. `RecordingOutput` keeps `OutputRecord`s (channel and rendered message) for
tests and replay (`list_records`, `take_records`), and `NoopOutput` discards everything.

## Keep parallel output in input order

Workers finish in any order. `OutputSequence` gives each unit of work an `OutputSlot` at its input position; messages
sent through the slot are held until the slot is dropped and every earlier position has been released, then written in
position order. Early positions that finish first print at once.

```rust
use std::sync::Arc;

use xrf_output::{OutputOptions, OutputSequence, OutputSlot, OutputVerbosity, RecordingOutput};

let sink: Arc<RecordingOutput> = Arc::new(RecordingOutput::default());
let output: OutputOptions = OutputOptions::new(sink.clone(), OutputVerbosity::Normal);
let sequence: OutputSequence = OutputSequence::new(&output, 2);

let second: OutputSlot = sequence.new_slot(1);
second.get_output().info("second");
drop(second);

// Position 0 is still open, so the second message is held.
assert!(sink.list_records().is_empty());

let first: OutputSlot = sequence.new_slot(0);
first.get_output().info("first");
drop(first);

let messages: Vec<String> = sink.list_records().iter().map(|it| it.get_message().to_string()).collect();

assert_eq!(messages, ["first", "second"]);
```

Take the slot as the first thing a unit of work does, before anything that can return early: a position nobody takes
holds back every later one until the sequence is dropped, which then writes the rest in order. A slot commits on drop,
so a unit that fails or panics still releases its position. Slots inherit the sequence's verbosity, so muted messages
are never held. The sequence orders output only; it does not schedule or limit the workers.

## Checks and related crates

Run `cargo test --locked -p xrf-output` from the repository root. Output carries messages, not results: a failure
message neither returns an error nor marks a report as failed. Progress and cancellation belong to
[xrf-job](../xrf-job/README.md); final findings belong to [xrf-report](../xrf-report/README.md).

See the [options](src/options.rs), [sinks](src/output.rs) and [ordered output](src/sequence.rs).
