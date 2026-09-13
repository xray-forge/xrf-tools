# xrf-gamedata

Validates assembled X-Ray game data across configs and the assets they reference, reading loose files and archives
through one VFS.

## Verify a project

```rust,no_run
use xrf_gamedata::{
  GamedataProject, GamedataProjectReadOptions, GamedataProjectVerifyOptions, GamedataVerificationType,
};

fn main() -> xrf_error::XrfResult {
  let read = GamedataProjectReadOptions {
    root: "gamedata".into(),
    ..Default::default()
  };

  let project = GamedataProject::open(&read)?;
  let options = GamedataProjectVerifyOptions {
    checks: vec![GamedataVerificationType::Ltx, GamedataVerificationType::Textures],
    ..Default::default()
  };

  let report = project.verify(&options)?;

  println!(
    "Status: {:?}; execution: {:?}",
    report.get_status(),
    report.get_outcome()
  );

  for check in report.get_checks() {
    println!("{:?}", check.get_status());
  }

  Ok(())
}
```

An `Ok` report can contain failed checks. Inspect the report's verdict and execution outcome: cancellation can leave
checks unperformed, and their silence is not a pass.

## Mount and select inputs

Set `GamedataProjectReadOptions::root`, then call `GamedataProject::open` for automatic mounting or `open_with_mode`
for an explicit `XrayMountMode`. Opening requires the root game config, `configs\system.ltx`.

Call `verify` with `GamedataProjectVerifyOptions`. Populate `checks` with the desired `GamedataVerificationType`
values. Options also carry strictness, live output, and a job handle for progress and cancellation.

Standard LTX is the default. To check a DLTX installation, supply its dialect in the read options before opening.
Inspect `skipped_mounts` and `collisions` when diagnosing incomplete or conflicting inputs.

## Check selection and results

Use `GamedataVerificationType::get_all()` for all selectable checks. The set covers animations, levels, LTX, meshes,
particles and their usage, scripts, shaders, sounds, spawns, textures, weapons, and weathers. Passing an empty selection
is an error; default verify options do not mean all checks.

Input coverage and name collisions run in addition to the selected checks. Coverage reports sources that could not
be mounted; collisions report names a mounted source holds but the engine cannot reach. Neither is replaced by
checking only the assets that happened to load.

The report keeps per-check findings, status, summary, and outcome. A failed content check and a failure to perform a
check are different verdicts. Preserve that distinction when converting the result to an exit code or UI message.

## Progress and execution

Pass a job handle through the verify options for nested progress and cooperative cancellation. Live messages go through
`OutputOptions`. The crate uses the caller's Rayon pool; an execution limit belongs around the whole operation, as
described in [xrf-job](../xrf-job/README.md).

Read tracing is optional in `GamedataProjectReadOptions`. Enable it for diagnosing repeated asset reads, then inspect
`get_read_trace_summary`; it adds accounting on the read path and is off by default.

## Errors and checks

Opening can fail before any report exists, including when the required config cannot be read. Verification returns
reports for check results, but an invalid invocation can still return `Err`. This crate validates the available static
data; it does not launch the game or execute its scripts.

Run `cargo test --locked -p xrf-gamedata` from the repository root.

See the [opening API](src/project/gamedata_project.rs), [options](src/project/gamedata_project_options.rs),
[verification API](src/project/gamedata_project_verify.rs), and [check selection](src/project/gamedata_verification_type.rs).
