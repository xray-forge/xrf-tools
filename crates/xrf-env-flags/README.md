# xrf-env-flags

Every environment variable the XRF workspace reads or honours, in one table (`EnvFlags::ALL`): what each takes, what
applies while it is unset, who reads it, and what it does. The table is the only way the workspace's own code reads a
runtime variable, and it writes the repository's [ENVIRONMENT.md](../../ENVIRONMENT.md).

## Read a variable

Each variable is an associated constant of `EnvFlags`, read as its value kind:

```rust,no_run
use xrf_env_flags::EnvFlags;

let backend: Option<&'static str> = EnvFlags::RENDER_BACKEND.read_choice();
let is_ray_query_off: bool = EnvFlags::RENDER_RAY_QUERY.read_switch() == Some(false);
```

A switch takes `1`, `true`, `on`, `yes` or `0`, `false`, `off`, `no`; a choice takes its names; both in any case. A value
the variable does not take is warned of in the log and reads as unset, so the default applies.

## Add a variable

1. Add a constant to `EnvFlags`, named `XRF_<AREA>_<NAME>` for the workspace's own, and list it in `EnvFlags::ALL`.
2. Read it through the constant where it is needed.
3. Run `cargo test -p xrf-env-flags`: it rewrites a stale `ENVIRONMENT.md` and fails once; commit the file.

Test-only and benchmark-only variables stay with their tests.

## Report the variables

`EnvFlags::list_readings` answers every variable a running binary reads, set or not, as `EnvFlagReading`: the desktop
app logs the set ones at startup and lists them in Settings, About. `EnvFlags::list_read_by` answers the ones one binary
reads, for its help.
