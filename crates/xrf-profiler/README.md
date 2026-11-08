# xrf-profiler

The Tracy client every XRF binary streams its zones to, and the features that turn it on. Nothing here runs in a normal
build: without `tracy`, [`start`] and [`send_message`] do nothing and every `profiling` scope compiles away.

## Instrument code

A crate marks its work with the [`profiling`](https://docs.rs/profiling) crate it depends on plainly, with no feature:
`profiling::scope!("vfs mount")`, `profiling::scope!("gamedata check", name)` with a detail, or
`#[profiling::function]`. Cargo unifies features across a build, so when a binary turns on `tracy` here, which turns on
`profiling/profile-with-tracy`, every scope in every crate of that binary becomes a zone.

Keep scopes coarse: a mount, a resolution, one part of a level open, one check of a verification, a document's open
and close. Never one per record, vertex or field: millions of zones swamp a capture and slow what they time. Where the
work is under about a tenth of a millisecond, leave it to the enclosing scope. Worker threads are already named
(`xrf-worker-<n>`, `xrf-once`), and the capture shows them by name.

## Capture

A binary calls [`start`] first thing in `main`, before any thread opens a zone. Build with the `tracy` profile, which
inherits `profiling` (release code with line tables) in its own target folder:

```text
cargo build --profile tracy -p xrf-cli --features tracy
cargo build --profile tracy -p xrf-app --features tracy
```

Then connect the Tracy 0.14 profiler and run. A `tracy` build is for reading a timeline, never for timing a whole
command: the client costs about half a second and a few hundred megabytes of commit when it starts, whatever its
features, though the zones themselves cost little. `tracy-memory` adds every allocation and free with a 12-frame call stack:
use it to find what holds memory, never to time, as it slows every allocation. It installs the build's global
allocator, so no other crate in that binary may declare one.
