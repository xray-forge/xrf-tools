# xrf-job

Progress reporting, cooperative cancellation and worker limits for long-running XRF operations.

An operation takes a `JobHandle` (usually in its options struct) and runs on an installed thread pool. The CLI and the
desktop app decide where progress goes, when to cancel, and how many workers to use.

Live messages belong to [xrf-output](../xrf-output/README.md), and final findings to [xrf-report](../xrf-report/README.md).

## Report progress

A `JobHandle` is one run's progress and cancellation. Clones share the run: the app gives one clone to the thread doing
the work and keeps another to call `cancel` on.

| Constructor                       | Use                                                                               |
| --------------------------------- | --------------------------------------------------------------------------------- |
| `JobHandle::inert()` or `default` | Reports nowhere but can still be cancelled; the CLI and most tests use it         |
| `JobHandle::new(sink)`            | Reports to a `ProgressSink` at most once per `DEFAULT_PROGRESS_INTERVAL` (100 ms) |
| `JobHandle::with_interval(..)`    | The same with another interval; `Duration::ZERO` reports every unit, for tests    |

The operation enters levels, counts work, and checks for cancellation between units:

```rust
use xrf_job::{JobHandle, JobScope};

# fn entries() -> Vec<u32> { vec![1, 2, 3] }
# fn write_entry(_: u32) {}
# fn main() -> xrf_error::XrfResult {
let job: JobHandle = JobHandle::inert();
let writing: JobScope = job.enter("write", Some(entries().len() as u64));

for entry in entries() {
  // Between entries, never inside one: a write already started cannot be halved.
  job.check_cancelled()?;

  write_entry(entry);
  writing.advance();
}
# Ok(())
# }
```

### Scopes

- `enter(id, total)` pushes a level and returns a `JobScope`, which leaves the level when dropped. Pass `None` for an
  unknown total.
- `enter_bytes` enters a level counting bytes (`ProgressUnit::Bytes`).
- A scope counts with `advance` or `advance_by(units)`, nests with its own `enter`, and reads its count with
  `get_completed`.
- Scopes are `Send` and `Sync`, so a pool's workers can all advance one level.
- `set_detail` names what a sequential job is working on; leave it empty for parallel work.

### Snapshots

Each `JobProgress` snapshot carries the whole active stack (outermost first), the `duration` since the handle was
created, and the current `detail`.

- Each level counts what its owner advances: checks for a parent, units for a leaf. Leaving a child adds nothing to its
  parent.
- The stack shows nesting, not siblings. Work running beside other work in one job uses `without_progress`: a handle
  sharing the job's cancellation and clock whose levels reach no sink.
- Entering and leaving a level report at once. `advance` reports only when the interval has passed, from whichever
  worker gets there first, so a sink is called from any thread and blocking in it blocks that worker.
- `report_if_due` lets a phase with nothing to count still report; `get_progress` reads the current snapshot.

### Sinks

- `NoopSink` discards snapshots.
- `RecordingSink` keeps every snapshot for tests (`list_reported`, `take_reported`).
- Consumers write their own: the CLI's `TerminalProgressSink` writes a line to stderr at a coarse interval; the app
  forwards snapshots to the window.

## Cancel

Cancellation is cooperative; nothing is interrupted or rolled back.

- `cancel` sets a flag.
- `is_cancelled` reads it, before expensive work.
- `check_cancelled` returns `XrfError::Cancelled`, so a loop or a parallel `try_for_each` stops with `?`.

The operation catches its own `Cancelled` and returns its normal result with `outcome` set to `JobOutcome::Cancelled`,
so the caller still learns which files were written. That result's counts describe the work done before the stop, not
a finished total.

## Limit workers

`ExecutionRequest` is what a caller asked for, parsed from text:

- `auto` (any case);
- a worker count from 1 to `ExecutionRequest::MAX_WORKERS` (1024);
- a share of the host, from `1%` to `100%`.

Anything else is `XrfError::Invalid`, naming the accepted forms.

`resolve` turns it into an `ExecutionPlan` for this host. `auto` uses `std::thread::available_parallelism`; a share
rounds down, to at least one worker. `get_origin` says whether the count was `ExecutionOrigin::Auto` or `Requested`.

```rust
use std::str::FromStr;

use xrf_job::{ExecutionOrigin, ExecutionPlan, ExecutionRequest};

# fn main() -> xrf_error::XrfResult {
// What a command line said, parsed without asking the machine anything.
let requested: ExecutionRequest = ExecutionRequest::from_str("50%")?;

// What it means on this host.
let plan: ExecutionPlan = requested.resolve();

assert!(plan.get_workers().get() >= 1);
assert_eq!(plan.get_origin(), ExecutionOrigin::Requested);
# Ok(())
# }
```

The plan's worker count is a ceiling for the whole operation.

- With the `rayon` feature, a binary runs one operation in `plan.install(..)`, or keeps one pool from
  `plan.build_pool()` and installs every operation into it. Nested Rayon work, including a dependency's, inherits that
  pool.
- Library crates call `par_iter` on whatever pool their caller installed and build none of their own.
- An operation may use fewer workers than planned only under `ExecutionOrigin::Auto`, and only when a measurement
  justifies it. A `Requested` count was chosen by a person.

### `PoolOnceLock`

`PoolOnceLock` (feature `rayon`) is a value built once that pool workers may wait on while its build fans out over
Rayon.

A plain `OnceLock` deadlocks there: the worker building it takes other jobs while waiting in its own `join`, and one of
those waits on the lock it is filling. `PoolOnceLock::get_or_init` builds on a one-thread pool of its own while the
caller's worker sleeps, so the build stays within the plan but is not parallel.

Use a plain `OnceLock` for a value whose build never reaches Rayon, since each `PoolOnceLock` build starts a thread.

## Features

- `rayon`: `ExecutionPlan::install`, `build_pool`, and `PoolOnceLock`.
- `typescript-bindings`: Specta types for the snapshot, outcome and plan types. `JobProgress` sends `duration` as whole
  milliseconds.

## Checks

Run `cargo test --locked -p xrf-job --features rayon` from the repository root.

Sources: the [handle](src/job/job_handle.rs), [scope](src/job/job_scope.rs) and
[execution plan](src/execution/execution_plan.rs). They are grouped into `job/`, `progress/` and `execution/`; every
type is exported from the crate root.
