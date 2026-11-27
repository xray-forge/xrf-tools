# xrf-game

The native game binary. It starts as the earliest prototype: today it logs and resolves its settings, and the window,
the level and the first movement mode come next. Its library holds how this program is put together (arguments,
directories, logging), kept apart from `main` so it is tested; what it draws with is
[xrf-game-settings](../settings/README.md)'.

## Running

```powershell
cargo run -p xrf-game -- --preset high --set view.shadows.resolution=2048 --set window.mode=windowed
```

- `--preset <name>` starts from a preset in place of the settings file's.
- `--set <path>=<value>`, repeated, changes one setting for this run, over the settings file, never written back.

## Files

- Its own files sit under `appdata/xrf-game/` beside the executable when that directory accepts files, else in its
  local data directory (`com.xrf.game`): `settings.ltx` among them (`GameDirectories`).
- Its logs are `xrf-game.<UTC start>.<process>.log` in the `appdata/logs/` it shares with the desktop app, through
  [xrf-logging](../../../crates/xrf-logging/README.md) (`GameLogSetup`).

## Checks

Run `cargo test --locked -p xrf-game` from the repository root.
