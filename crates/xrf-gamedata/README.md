# xrf-gamedata

Validates assembled X-Ray game data across configs and the assets they reference, reading loose files and archives
through one VFS.

A `GamedataProject` is a game data tree, or a whole installation with its archive volumes, opened as the engine would
see it. Each check reads what one kind of asset needs (configs, meshes, textures, levels, scripts and so on) through
the owning format crate and reports findings by rule. `xrf-cli gamedata verify` and the desktop app's verifier are
built on it. The crate validates static data; it does not launch the game or execute its scripts.

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
    println!(
      "{}: {:?}, {} findings",
      check.get_verification_type(),
      check.get_status(),
      check.get_findings().len()
    );
  }

  Ok(())
}
```

An `Ok` report can contain failed checks. Inspect the report's verdict and execution outcome, or `is_valid`, which
holds only for a completed run that passed: cancellation can leave checks unperformed, and their silence is not a pass.

## Open a project

Set `GamedataProjectReadOptions::root`, then call `GamedataProject::open`, which treats the path as an installation
only when it declares one (`XrayMountMode::Auto`), or `open_with_mode` for an explicit `XrayMountMode`. The root must be
an existing directory, and something in it must resolve `configs\system.ltx`; otherwise opening fails before any report
exists. `ignored` takes logical prefixes that loose directories leave out of every check (archive volumes are
unaffected), without changing what identifies the project.

The read options also choose how configs are read:

- `dialect`: standard LTX by default. Supply [xrf-dltx](../xrf-dltx/README.md)'s dialect to check a DLTX installation.
- `engine`: an `XrayEngineChoice` ([xrf-engine-target](../xrf-engine-target/README.md)), detected from the tree on
  `auto`, the default. `get_engine` answers the engine chosen and what decided it; the environment check reads configs
  as that engine.
- `is_tracing_reads`: accounts for every asset physically read, for diagnosing repeated reads through
  `get_read_trace_summary`. It adds work on the read path and is off by default.

The project's config tree is an [xrf-ltx](../xrf-ltx/README.md) `LtxProject` over the same search as every check, with
section schemes loaded and resolutions cached, so checks that read `system.ltx` share one resolution. The VFS retains
what its verification cache policy keeps (motions among them, so a shared animation bank is read once);
`get_cache_stats` reports how well that served. Inspect `skipped_mounts` and `collisions` when diagnosing incomplete or
conflicting inputs.

## Select checks

`GamedataProjectVerifyOptions::checks` names the `GamedataVerificationType`s to run; `GamedataVerificationType::ALL`
and `get_all()` list every selectable check, and each parses from its CLI name:

- `animations`: HUD item and player HUD animations, motion collisions, object motions, and post-process effects.
- `environment`: everything under `configs\environment` the engine reads, and the textures its keyframes name.
- `levels`: compiled level bundles, their companion files, the level roster, and the game materials collision faces
  are made of.
- `ltx`: config resolution, section schemes, and formatting.
- `materials`: the game material library, `gamemtl.xr`, against what the engine asserts on while loading it, the
  materials the engine looks up by names of its own, wallmark textures, and materials configs and climable spawn
  objects name.
- `meshes`: OGF visuals, their motions, shaders, and the game materials their bones are made of.
- `particles`: the particle library and its textures.
- `particles-usage`: particle references from configs, spawns and material pairs.
- `scripts`: script paths, reads and syntax.
- `shaders`: shader sources, includes and Lua shader scripts.
- `sounds`: sound files and the references to them, material pairs' among them.
- `spawns`: spawn files.
- `textures`: DDS textures, `.thm` descriptors and bump companions.
- `weapons`: weapon sections.

Only `materials` fails on a tree without a readable material library; `levels`, `meshes`, `particles-usage` and
`sounds` then leave out what they would resolve against it.

Passing an empty selection is an error; default verify options do not mean all checks. A check named twice runs once,
in first-requested order. The options also carry strictness, live output, and a job handle.

Input coverage and name collisions always run first, in addition to the selected checks, and are not selectable.
Coverage reports sources the installation declared that could not be opened; collisions report files a mounted source
holds but the engine cannot reach, because another file claims their engine identity. Neither is replaced by checking
only the assets that happened to load.

## Read the report

`GamedataVerificationReport` keeps one `GamedataVerificationCheckReport` per check, in selection order however the
checks were scheduled: its status, findings, summary, and duration. Each finding carries a stable rule id prefixed by
its check, such as `scripts.syntax` or `animations.post-process-value` (`GamedataVerificationRule`). `get_status`
aggregates the checks; `to_report` converts to the shared `xrf_report::Report` model, without the per-check summaries.

A failed content check (`Failed`) and a failure to perform a check (`Error`, with a `checks.execution` finding) are
different verdicts, and a check stopped by cancellation is `Incomplete`. Preserve those distinctions when converting
the result to an exit code or UI message.

## Progress and execution

Pass a job handle through the verify options for nested progress and cooperative cancellation; after cancellation no
further check starts, and running checks stop at their own iteration boundaries. Checks run together on the caller's
Rayon pool, a few at a time while output is being read, all at once when output is silent; an execution limit belongs
around the whole operation, as described in [xrf-job](../xrf-job/README.md). Live messages go through `OutputOptions`,
and each check's output is released in selection order, so a run says the same things in the same order however its
workers were scheduled.

## Errors and checks

Opening can fail before any report exists. `verify` returns `Err` only for an invalid invocation, such as an empty
selection; a check that fails to run becomes an `Error` check report instead.

Run `cargo test --locked -p xrf-gamedata` from the repository root.

See the [opening API](src/project/gamedata_project.rs), options for [reading](src/project/gamedata_project_read_options.rs)
and [verifying](src/project/gamedata_project_verify_options.rs),
[verification API](src/project/gamedata_project_verify.rs),
[check selection](src/project/gamedata_verification_type.rs), and [rule ids](src/project/gamedata_verification_rule.rs).
