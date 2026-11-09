# xrf-gamedata

Validates assembled X-Ray game data: configs and the assets they reference, read from loose files and archives through
one VFS.

A `GamedataProject` is a game data tree, or a whole installation with its archives, opened as the engine would see it.
Each check reads one kind of asset (configs, meshes, textures, levels, scripts and so on) through its format crate and
reports findings by rule.

- `xrf-cli gamedata verify` and the desktop app's verifier are built on it.
- It checks static data only. It does not launch the game or run its scripts.

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

An `Ok` report can still hold failed checks. Look at the verdict and the execution outcome, or call `is_valid`, which
is true only for a completed run that passed. A cancelled run can leave checks unperformed, and their silence is not a
pass.

## Open a project

Set `GamedataProjectReadOptions::root`, then call:

- `GamedataProject::open`, which treats the path as an installation only when it declares one (`XrayMountMode::Auto`);
- or `open_with_mode`, for an explicit `XrayMountMode`.

Opening fails, before any report exists, unless the root is an existing directory in which `configs\system.ltx`
resolves.

Read options:

| Option             | Effect                                                                                         |
| ------------------ | ---------------------------------------------------------------------------------------------- |
| `ignored`          | Logical prefixes left out of every check in loose directories; archives are unaffected         |
| `dialect`          | Standard LTX by default; pass [xrf-dltx](../xrf-dltx/README.md)'s dialect for a DLTX install   |
| `engine`           | An `XrayEngineChoice` ([xrf-engine-target](../xrf-engine-target/README.md)); `auto` detects it |
| `is_tracing_reads` | Records every physical read, for `get_read_trace_summary`. Off by default; it costs read time  |

`get_engine` answers the engine chosen and what decided it; the environment check reads configs as that engine.

Shared state:

- The config tree is one [xrf-ltx](../xrf-ltx/README.md) `LtxProject` over the same search as every check, with section
  schemes loaded and resolutions cached, so checks reading `system.ltx` share one resolution.
- The VFS keeps what its verification cache policy retains (motions among them, so a shared animation bank is read
  once). `get_cache_stats` reports how well that worked.
- `skipped_mounts` and `collisions` explain incomplete or conflicting inputs.

## Select checks

`GamedataProjectVerifyOptions::checks` names the `GamedataVerificationType`s to run. `GamedataVerificationType::ALL`
and `get_all()` list them; each parses from its CLI name.

| Check             | Covers                                                                                                                                                     |
| ----------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `animations`      | HUD and player HUD animations, motion collisions, object motions, post-process effects                                                                     |
| `environment`     | Everything under `configs\environment` the engine reads, and the textures its keyframes name                                                               |
| `levels`          | Compiled level bundles, their companion files, the level roster, collision face materials                                                                  |
| `ltx`             | Config resolution, section schemes, and formatting                                                                                                         |
| `materials`       | `gamemtl.xr` against the engine's load-time asserts, engine-named materials, wallmark textures, and materials named by configs and climbable spawn objects |
| `meshes`          | OGF visuals, their motions, shaders, and bone materials                                                                                                    |
| `particles`       | The particle library and its textures                                                                                                                      |
| `particles-usage` | Particle references from configs, spawns and material pairs                                                                                                |
| `scripts`         | Script paths, reads and syntax                                                                                                                             |
| `shaders`         | Shader sources, includes and Lua shader scripts                                                                                                            |
| `sounds`          | Sound files and references to them, including material pairs'                                                                                              |
| `spawns`          | Spawn files                                                                                                                                                |
| `textures`        | DDS textures, `.thm` descriptors and bump companions                                                                                                       |
| `weapons`         | Weapon sections                                                                                                                                            |

- An empty selection is an error. Default options do not mean "all checks".
- A check named twice runs once, at its first position.
- Without a readable material library, only `materials` fails; `levels`, `meshes`, `particles-usage` and `sounds` skip
  what they would resolve against it.
- The options also carry strictness, live output, and a job handle.

Two checks always run first and cannot be deselected:

- **Input coverage** reports sources the installation declares but that could not be opened.
- **Name collisions** report files a source holds that the engine cannot reach, because another file claims their
  engine identity.

## Read the report

`GamedataVerificationReport` keeps one `GamedataVerificationCheckReport` per check, in selection order: status,
findings, summary and duration.

- Each finding has a stable rule id prefixed by its check, such as `scripts.syntax` or `animations.post-process-value`
  (`GamedataVerificationRule`).
- `get_status` aggregates the checks.
- `to_report` converts to the shared `xrf_report::Report` model, without the per-check summaries.

Keep these verdicts apart when turning a result into an exit code or a UI message:

| Status       | Meaning                                                        |
| ------------ | -------------------------------------------------------------- |
| `Failed`     | The check ran and found problems                               |
| `Error`      | The check could not run; a `checks.execution` finding says why |
| `Incomplete` | Cancellation stopped the check                                 |

## How checks are built

- `verify` is the only entry point. Each check is a crate-private verifier type (`ScriptsVerifier`, `WeaponsVerifier`
  and so on) built through one trait; `GamedataVerificationType::run` builds and runs any of them.
- The project lends each check only what all checks share: the search, the config tree, the spawn files and the
  material library.
- A check over many independent assets runs one sweep: it verifies each in parallel, releases output in item order,
  counts items with findings, and orders findings by asset.
- Every problem is a finding naming what is wrong and where, never a logged reason with a bare "invalid".

## Progress and execution

- Pass a job handle in the verify options for progress and cancellation. After cancellation, no further check starts
  and running checks stop at their next iteration.
- Progress is one `checks` level, advanced per finished check. A check running alone reports its own levels beneath it;
  checks running together report none.
- Checks run together on the caller's Rayon pool: a few at a time while output is being read, all at once when output
  is silent. Put an execution limit around the whole operation, as [xrf-job](../xrf-job/README.md) describes.
- Live messages go through `OutputOptions`. Each check's output is released in selection order, so a run reports the
  same things in the same order however its workers were scheduled.

## Errors

- Opening can fail before any report exists.
- `verify` returns `Err` only for an invalid call, such as an empty selection. A check that cannot run becomes an
  `Error` check report instead.

## Checks

Run `cargo test --locked -p xrf-gamedata` from the repository root.

Sources: the [opening API](src/project/gamedata_project.rs), options for
[reading](src/project/gamedata_project_read_options.rs) and
[verifying](src/project/gamedata_project_verify_options.rs), the
[verification API](src/project/gamedata_project_verify.rs),
[check selection](src/project/gamedata_verification_type.rs), the [check trait](src/project/gamedata_check.rs) and
[sweep](src/project/gamedata_asset_sweep.rs), and [rule ids](src/project/gamedata_verification_rule.rs).
