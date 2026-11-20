# xrf-profiler

Starts the Tracy client every XRF binary streams its profiling zones to, and holds the features that turn it on.

Nothing here runs in a normal build. Without the `tracy` feature, [`start`] and [`send_message`] do nothing and every
`profiling` scope compiles away.

## Instrument code

A crate marks its work with the [`profiling`](https://docs.rs/profiling) crate, depended on plainly with no feature:

- `profiling::scope!("vfs mount")`;
- `profiling::scope!("gamedata check", name)`, with a detail;
- `#[profiling::function]`.

Cargo unifies features across a build. When a binary turns on `tracy` here (which turns on
`profiling/profile-with-tracy`), every scope in every crate of that binary becomes a zone.

Keep scopes coarse:

- Good: a mount, a resolution, one part of opening a level, one check of a verification, a document's open and close.
- Never one per record, vertex or field: millions of zones swamp a capture and slow what they measure.
- Leave work under about a tenth of a millisecond to the enclosing scope.

Worker threads are already named (`xrf-worker-<n>`, `xrf-once`), and the capture shows them by name.

## Capture

1. Call [`start`] first thing in `main`, before any thread opens a zone.
2. Build with the `tracy` profile, which inherits `profiling` (release code with line tables) in its own target folder:

   ```text
   cargo build --profile tracy -p xrf-cli --features tracy
   cargo build --profile tracy -p xrf-app --features tracy
   ```

3. Connect the Tracy 0.14 profiler, then run.

A `tracy` build is for reading a timeline, not for timing a whole command. Starting the client costs about half a
second and a few hundred megabytes of committed memory, whatever the features; the zones themselves are cheap.

`tracy-memory` also records every allocation and free with a 12-frame call stack.

- Use it to find what holds memory, never for timing: it slows every allocation.
- It installs the build's global allocator, so no other crate in that binary may declare one.
