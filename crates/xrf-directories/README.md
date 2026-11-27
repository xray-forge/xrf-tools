# xrf-directories

Where a native application keeps its own files: `appdata` beside its executable when that directory accepts files, as
a portable install keeps everything together and X-Ray keeps its own `appdata`, else its own local data directory.
The desktop app and the game both resolve their folders through it.

## Use

- `DataDirectories::for_identifier(identifier)` takes the application's local data directory from its bundle
  identifier (`com.xrf.tools`, `com.xrf.game`), so the folders are known before anything else runs; `new(local_data)`
  names one directly.
- `get_app_data_directories()` lists where the application's own files may go, best first, and
  `get_logs_directories()` the `logs` folder in each, which applications sharing an `appdata` share.
- `get_writable_executable_directory()` decides once a run whether the executable's directory accepts files, by
  writing and removing a probe of the process's own, so every caller addresses the same folders.

What an application keeps in those folders (its preferences, settings, caches, saved files) is the application's own
to name.

## Checks

Run `cargo test --locked -p xrf-directories` from the repository root.
