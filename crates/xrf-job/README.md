# xrf-job

Progress reporting and cooperative cancellation for long-running XRF operations.

A crate that can take minutes over a set it did not choose — packing volumes, unpacking a tree, verifying a project —
reports through a `JobHandle` rather than growing its own counter and clock. The handle owns everything that must not
drift between operations: how units are counted across workers, how elapsed time is measured, and how often anything is
emitted. An operation calls two verbs, `enter` and `advance`, and checks for cancellation where stopping is safe.

Delivery is a `ProgressSink`, so the same operation reports to a desktop IPC channel, to nothing at all, or to a
recorder in a test, without knowing which.

```rust,no_run
use xrf_job::{JobHandle, JobScope};

# fn entries() -> Vec<u32> { vec![1, 2, 3] }
# fn write_entry(_: u32) {}
# fn main() -> xrf_error::XrfResult {

// The command line and tests take a handle that reports nowhere and is never cancelled.
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

Progress nests. A level's `completed` and `total` count whatever that level is made of — its child levels for a parent,
its own units for a leaf — so a run reporting `["verify" 2/7, "textures" 400/40000]` uses the same mechanism as one
reporting `["unpack" 1/2, "write" 45000/100000]`. Each emitted `JobProgress` is a snapshot of the whole active stack.

Totals are honest. A phase that cannot count its work reports `None` and is rendered as indeterminate rather than as a
fabricated percentage. A level whose units are bytes says so through `ProgressUnit`, because entry counts mislead badly
where entry sizes differ by four orders of magnitude.

Emission is bounded by time rather than by a count step, so a set of a hundred thousand small entries and a set of two
hundred large ones both report at a rate a person can read. One throttle governs the whole stack: a fast leaf cannot
multiply the rate. Phase transitions and the first update are never swallowed by it.

Cancellation is cooperative and never asynchronous. `check_cancelled` yields `XrfError::Cancelled`, which composes with
`?` and — unlike a boolean — can break a parallel iterator that has no other early exit. That error is control flow: an
operation catches its own cancellation and reports what it completed, because a caller needs to know what was written,
not merely that it stopped.
