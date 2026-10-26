# xrf-report

Immutable, deterministic reports of what XRF commands checked and found. A `Report` holds `CheckReport`s, each with a
verdict and its `Finding`s; the CLI's verify commands build one and serialize it into their JSON report.

## Build a report

```rust
use xrf_report::{CheckId, CheckReport, Finding, Report, RuleId, Status};

fn main() -> xrf_error::XrfResult {
  let finding: Finding = Finding::new(
    RuleId::new("textures.missing")?,
    Some("textures/example.dds".to_owned()),
    "Referenced texture was not found",
  );

  let check: CheckReport = CheckReport::new(CheckId::new("textures")?, Status::Failed, None, vec![finding]);
  let report: Report = Report::new(vec![check]);

  assert_eq!(report.status(), Status::Failed);
  assert_eq!(report.checks()[0].findings().len(), 1);

  Ok(())
}
```

## Concepts

- `Finding::new(rule_id, subject, message)`: one rule violation or checker error. The optional subject names the
  affected asset or value as report text, not a validated path; use `None` when the finding concerns the whole check.
- `CheckReport::new(id, status, duration, findings)`: one check with an explicit `Status`, an optional measured
  `Duration` (`None` when the check did not time itself), and its findings, sorted into their report order. That order
  is `Finding`'s `Ord`: by subject (`None` first), then rule, then message.
- `Report::new(checks)`: keeps the checks in the caller's order and aggregates their statuses.

`CheckId` names a check and `RuleId` one rule within it. Both reject an empty string with an `IdentifierError`, which
converts to `XrfError::Invalid` so `?` works in an `XrfResult` function. Neither imposes a naming scheme; keeping names
stable is the producer's job.

Nothing is inferred from findings: a check with findings can pass, and one without can fail. Supply the verdict, or use
`Status::from_is_valid(bool)` for a plain pass or fail.

## Status and serialization

`Status::aggregate` takes the strongest status present, in this order: `Error`, `Failed`, `Incomplete`, `Passed`,
`Skipped`. So a skipped check beside a passed one gives `Passed`, and an empty report is `Skipped`.

Reports, checks and findings serialize with camelCase field names in alphabetical order, as their fields are declared.
Statuses serialize as lowercase strings, identifiers as plain strings, and `duration` as whole milliseconds or `null`
when unmeasured. Because construction fixes the order, the output does not depend on which worker finished first. The
CLI's JSON reports, their e2e snapshots and the app's TypeScript mirrors read this shape, which the crate's
[serialization tests](src/tests.rs) pin.

## Checks and related crates

Run `cargo test --locked -p xrf-report` from the repository root. Building a report does not log, write a file, or
choose an exit code; those belong to the consuming command. Use [xrf-output](../xrf-output/README.md) for live
messages and [xrf-job](../xrf-job/README.md) for progress.

See the [report](src/report.rs), [check](src/check_report.rs), [finding](src/finding.rs), [status](src/status.rs), the
[check](src/check_id.rs) and [rule](src/rule_id.rs) identifiers and their [error](src/identifier_error.rs).
