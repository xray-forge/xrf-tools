# xrf-job

Progress reporting, cooperative cancellation, and worker limits for long-running XRF operations. An operation takes a
`JobHandle` (usually through its options struct) and an installed thread pool; the CLI and the desktop app decide where
progress goes, when to cancel, and how wide to run.

## Progress and cancellation

A `JobHandle` is one run's progress and cancellation together. Clones share the run: the app hands one clone to the
blocking thread doing the work and keeps another to call `cancel` on.

| Constructor                       | Use                                                                         |
| --------------------------------- | --------------------------------------------------------------------------- |
| `JobHandle::inert()` or `default` | Reports nowhere and is never cancelled; the CLI and most tests pass it      |
| `JobHandle::new(sink)`            | Reports to a `ProgressSink` at most once per `DEFAULT_PROGRESS_INTERVAL`    |
| `JobHandle::with_interval(..)`    | Same, with another interval; `Duration::ZERO` reports every unit, for tests |

`DEFAULT_PROGRESS_INTERVAL` is 100 ms; the desktop app uses it.

The operation enters levels and counts work, and checks for cancellation between units:

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

`enter(id, total)` pushes a level and returns a `JobScope`, which leaves the level when dropped. Use `None` for a total
not known in advance and `enter_bytes` for a level counting bytes (`ProgressUnit::Bytes`). A scope counts with `advance`
or `advance_by(units)`, nests with its own `enter`, and reads its count with `get_completed`. Scopes are `Send` and
`Sync` through shared state, so a pool's workers can all advance one level. `set_detail` names what a sequential job is
on; leave it empty for parallel work.

Each `JobProgress` snapshot carries the whole active stack, outermost first, plus `duration` since the handle was
created and the current `detail`. A parent level counts its finished children, a leaf its own units. Entering and
leaving a level are reported at once; `advance` reports only when the interval has passed, from whichever worker gets
there first, so a sink is called from arbitrary threads and blocking in it blocks that worker. `report_if_due` lets a
phase with nothing to count still report, and `get_progress` reads the current snapshot on demand.

Sinks: `NoopSink` discards, `RecordingSink` keeps every snapshot for tests (`list_reported`, `take_reported`), and
`LoggingSink` writes each as an `info` log line; give it a coarse interval.

Cancellation is cooperative. `cancel` sets a flag; `is_cancelled` reads it before expensive work, and `check_cancelled`
returns `XrfError::Cancelled` so a loop or a parallel `try_for_each` stops with `?`. Nothing is interrupted or rolled
back. The operation catches its own `Cancelled` and returns its normal result with an `outcome` of
`JobOutcome::Cancelled`, so the caller still learns which files were written; that result's counts describe the work
done before the stop, never a finished total.

## Execution limits

`ExecutionRequest` is what a caller asked for, parsed from text: `auto` (any case), a worker count from 1 to
`ExecutionRequest::MAX_WORKERS` (1024), or a share of the host from `1%` to `100%`. Anything else is `XrfError::Invalid`
with a message naming the accepted forms. `resolve` turns it into an `ExecutionPlan` for this host: `auto` uses
`std::thread::available_parallelism`, a share rounds down with a floor of one worker, and `get_origin` says whether the
count was `ExecutionOrigin::Auto` or `Requested`.

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

The plan's worker count is a ceiling on the whole operation. With the `rayon` feature, a binary runs one operation in
`plan.install(..)` or keeps one pool from `plan.build_pool()` and installs every operation into it; nested Rayon work,
including a dependency's, inherits that pool. Library crates call `par_iter` on whatever pool their caller installed
and build none of their own. An operation may use fewer workers than the plan only under `ExecutionOrigin::Auto`, and
only when a measurement justifies it; a `Requested` count was named by a person.

`PoolOnceLock` (feature `rayon`) is a value built once that pool workers may wait on while its build fans out over
Rayon. A plain `OnceLock` deadlocks there: the worker building it takes other jobs while it waits inside its own
`join`, and one of those waits on the lock that worker is filling. `PoolOnceLock::get_or_init` builds on a one-thread
pool of its own while the caller's worker sleeps, so the build stays within the plan but is not parallel. Keep a plain
`OnceLock` for a value whose build never reaches Rayon, since each build starts a thread.

The `typescript-bindings` feature derives Specta types for the snapshot, outcome and plan types; `JobProgress` sends
`duration` as whole milliseconds.

## Checks

Run `cargo test --locked -p xrf-job --features rayon` from the repository root; this page's examples run as doc tests.
See the [handle](src/job_handle.rs), [scope](src/job_scope.rs) and [execution plan](src/execution_plan.rs). Live
messages belong to [xrf-output](../xrf-output/README.md), and final findings to [xrf-report](../xrf-report/README.md).
