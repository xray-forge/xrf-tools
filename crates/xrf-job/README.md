# xrf-job

Provides progress reporting, cooperative cancellation, and execution limits for long-running XRF operations.

## Progress and cancellation

Pass a `JobHandle` to an operation. Enter phases with `enter`, count completed units with `advance`, and call
`check_cancelled` at safe stopping points. `JobScope` leaves its phase when dropped.

Use `JobHandle::new` with a `ProgressSink` for delivery, or `inert` when no progress is needed. `RecordingSink` captures
snapshots in tests. Cloned handles share the same job; call `cancel` on one to request a stop.

```rust,no_run
use xrf_job::{JobHandle, JobScope};

# fn entries() -> Vec<u32> { vec![1, 2, 3] }
# fn write_entry(_: u32) {}
# fn main() -> xrf_error::XrfResult {

// A handle that reports nowhere.
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

Progress nests: each `JobProgress` snapshot contains the active stack. Parent levels count completed children; leaf
levels count their own units. Use `None` for an unknown total and `enter_bytes` for byte counts.

Updates share one time throttle, while phase transitions are emitted immediately. `check_cancelled` returns
`XrfError::Cancelled`; it does not interrupt work or roll back writes. The operation decides where to stop and how to
report partial results.

## Execution

Parse `auto`, a worker count, or a percentage into `ExecutionRequest`. Its `resolve` method chooses a host-specific
`ExecutionPlan`, and `ExecutionOrigin` records whether the choice was automatic or requested.

```rust
use std::str::FromStr;
use xrf_job::{ExecutionOrigin, ExecutionPlan, ExecutionRequest};

# fn main() -> xrf_error::XrfResult {
    // What a command line said, parsed without asking the machine anything.
    let requested: ExecutionRequest = ExecutionRequest::from_str("50%")?;

    // What it means on this host.
    let plan: ExecutionPlan = requested.resolve();

    assert!(plan.get_workers().get() >= 1);
    // A person chose this width, so an operation may not quietly use less of it.
    assert_eq!(plan.get_origin(), ExecutionOrigin::Requested);
    # Ok(())
    #
}
```

With the `rayon` feature, wrap a single operation in `plan.install(..)`, or build a reusable pool with
`plan.build_pool()` and install operations into it. Nested Rayon work inherits that pool. Libraries use the caller's
pool rather than building their own; automatic plans may use fewer workers when measurements justify it.

See the [handle](src/job_handle.rs) and [execution plan](src/execution_plan.rs). Live messages belong to
[xrf-output](../xrf-output/README.md), and final findings to [xrf-report](../xrf-report/README.md).
