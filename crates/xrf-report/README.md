# xrf-report

Immutable, deterministic reports of what XRF commands checked and found.

A `Report` holds `CheckReport`s, each with a verdict and its `Finding`s. The CLI's verify commands build one and
serialize it into their JSON report.

## Build a report

```rust
use xrf_report::{CheckId, CheckReport, Finding, Report, RuleId, Status};

fn main() -> xrf_error::XrfResult {
  let missing: Finding = Finding::failure(
    RuleId::new("textures.missing")?,
    Some("textures/example.dds".to_owned()),
    "Referenced texture was not found",
  );
  let residue: Finding = Finding::warning(
    RuleId::new("textures.chunk-residue")?,
    Some("textures/other.thm".to_owned()),
    "Bytes the engine never reads",
  );

  let report: Report = Report::new(vec![
    CheckReport::judge(CheckId::new("textures")?, None, vec![missing]),
    CheckReport::judge(CheckId::new("descriptors")?, None, vec![residue]),
  ]);

  assert_eq!(report.status(), Status::Failed);
  assert_eq!(report.checks()[1].status(), Status::Passed);
  assert_eq!(report.into_strict().checks()[1].status(), Status::Failed);

  Ok(())
}
```

## Parts of a report

| Constructor                                        | Holds                                                       |
| -------------------------------------------------- | ----------------------------------------------------------- |
| `Finding::failure(rule_id, subject, message)`      | A rule violation or checker error that fails its check      |
| `Finding::warning(rule_id, subject, message)`      | One that is reported, and fails its check only when strict  |
| `CheckReport::judge(id, duration, findings)`       | A check whose verdict its findings decide                   |
| `CheckReport::new(id, status, duration, findings)` | A check whose verdict the producer supplies                 |
| `Report::new(checks)`                              | The checks in the caller's order, with an aggregated status |

- A finding's optional subject names the affected asset or value as text, not a validated path. Use `None` when the
  finding concerns the whole check.
- `duration` is `None` when the check did not time itself.
- Findings are sorted by `Finding`'s `Ord`: by subject (`None` first), then rule, then message, then failures first.
- `CheckId` names a check and `RuleId` a rule within it. Both reject an empty string with an `IdentifierError`, which
  converts to `XrfError::Invalid`, so `?` works in an `XrfResult` function. Neither imposes a naming scheme; keeping
  names stable is up to the producer.

## Severity

Each finding carries its `Severity`, which the producer decides because it knows what the defect costs: a `failure` is
what the consumer cannot work with (the engine asserts on it or refuses it, the file does not read), a `warning` what it
tolerates. A reader tells what failed a check from the findings alone, without knowing the check's rules.

`CheckReport::judge` fails a check on any failure and passes it with warnings only (`Status::from_findings`). A check
whose verdict depends on more than its findings, one that skipped, stopped or broke, supplies it through
`CheckReport::new`, or `Status::from_is_valid(bool)` for a plain pass or fail.

Strictness is applied to a finished report, never threaded through a check: `Report::into_strict` (and
`CheckReport::into_strict`) fails every check that passed with findings, leaving each finding's severity as it was and
every other verdict as it stood. A producer builds the lenient report and the command asked to be strict converts it.

## Status

`Status::aggregate` takes the strongest status present, in this order: `Error`, `Failed`, `Incomplete`, `Passed`,
`Skipped`. A skipped check beside a passed one gives `Passed`; an empty report is `Skipped`.

## Serialization

- Reports, checks and findings serialize with camelCase field names, in alphabetical order (as declared).
- Statuses and severities are lowercase strings, identifiers plain strings, and `duration` whole milliseconds or `null`.
- Construction fixes the order, so the output does not depend on which worker finished first.

The CLI's JSON reports, their e2e snapshots and the app's TypeScript mirrors read this shape, which the
[serialization tests](src/tests.rs) pin.

Building a report does not log, write a file, or choose an exit code; the consuming command does. Use
[xrf-output](../xrf-output/README.md) for live messages and [xrf-job](../xrf-job/README.md) for progress.

## Checks

Run `cargo test --locked -p xrf-report` from the repository root.

Sources: the [report](src/report.rs), [check](src/check_report.rs), [finding](src/finding.rs),
[severity](src/severity.rs), [status](src/status.rs), the [check](src/check_id.rs) and [rule](src/rule_id.rs)
identifiers and their
[error](src/identifier_error.rs).
