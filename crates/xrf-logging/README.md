# xrf-logging

How a native application logs: every record to the console and to the run's own log file, panics included, so a
release build, which has no console, still leaves one behind. The desktop app and the game both log through it, each
under its own name.

## Starting

`LogSetup { application, version, directories, tail }.install(&filter)` installs the process's logger once, before
anything logs, and answers the run (`LogRun`), none where no directory took its log. A `LogTail` given as `tail` also
keeps the records it passes, at its level or above, as short lines (`LEVEL message`) an application's own viewer takes,
as the game's console does; it holds a bounded number not yet taken, the oldest let go first. `LogFilter::from_environment()` reads
`RUST_LOG`, else the build's default level (debug in a debug build, info in a release one) with naga and wgpu held to
warnings, whose debug lines run to millions a shader compile and stall the render thread; a host reading its own
records apart from the logger, as the app's UI log does, filters them by the same `LogFilter`.

## Runs and their files

- A run's log is `<application>.<UTC start>.<process>.log`, such as `xrf-app.2026-11-22T10-30-05Z.4242.log`, made in
  the first of `directories` that takes it. The start, to the second, sorts as time does; the process tells two runs of
  one second apart.
- `LogRun::create_companion(suffix)` makes a file of the run beside its log, such as the app's
  `xrf-app.<start>.<process>.ui.log`.
- A directory keeps the last `LogRun::KEPT_RUNS` runs of the application, each log and its companions pruned together.
  Only the exact names a run writes are pruned: another application's runs and the single logs earlier versions wrote
  stay. A file a running instance holds stays too; on Windows a log is opened without delete sharing.
- Each record is written as it comes, unbuffered, so a log read after a freeze or a crash holds everything before it.

## Lines and the clock

`LogLayout` lays every line out as env_logger's default format does, stamped to the millisecond (`LogTimestamp`), so
a log and its companions read side by side. `ProcessStart::stamp()`, called first in `main`, fixes when the process
began, for the dates (`get_started_at`) and the uptime (`get_uptime`) a host reports.

## Checks

Run `cargo test --locked -p xrf-logging` from the repository root.
