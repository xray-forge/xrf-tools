# xrf-report

Immutable, deterministic reports of what XRF commands checked and found.

A `Report` holds `CheckReport`s, each with a verdict and its `Finding`s. The CLI's verify commands build one and
serialize it into their JSON report.

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

## Parts of a report

| Constructor                                        | Holds                                                       |
| -------------------------------------------------- | ----------------------------------------------------------- |
| `Finding::new(rule_id, subject, message)`          | One rule violation or checker error                         |
| `CheckReport::new(id, status, duration, findings)` | One check: its verdict, how long it took, and its findings  |
| `Report::new(checks)`                              | The checks in the caller's order, with an aggregated status |

- A finding's optional subject names the affected asset or value as text, not a validated path. Use `None` when the
  finding concerns the whole check.
- `duration` is `None` when the check did not time itself.
- Findings are sorted by `Finding`'s `Ord`: by subject (`None` first), then rule, then message.
- `CheckId` names a check and `RuleId` a rule within it. Both reject an empty string with an `IdentifierError`, which
  converts to `XrfError::Invalid`, so `?` works in an `XrfResult` function. Neither imposes a naming scheme; keeping
  names stable is up to the producer.

Nothing is inferred from findings: a check with findings can pass, and one without can fail. Supply the verdict, or use
`Status::from_is_valid(bool)` for a plain pass or fail.

## Status

`Status::aggregate` takes the strongest status present, in this order: `Error`, `Failed`, `Incomplete`, `Passed`,
`Skipped`. A skipped check beside a passed one gives `Passed`; an empty report is `Skipped`.

## Serialization

- Reports, checks and findings serialize with camelCase field names, in alphabetical order (as declared).
- Statuses are lowercase strings, identifiers plain strings, and `duration` whole milliseconds or `null`.
- Construction fixes the order, so the output does not depend on which worker finished first.

The CLI's JSON reports, their e2e snapshots and the app's TypeScript mirrors read this shape, which the
[serialization tests](src/tests.rs) pin.

Building a report does not log, write a file, or choose an exit code; the consuming command does. Use
[xrf-output](../xrf-output/README.md) for live messages and [xrf-job](../xrf-job/README.md) for progress.

## Checks

Run `cargo test --locked -p xrf-report` from the repository root.

Sources: the [report](src/report.rs), [check](src/check_report.rs), [finding](src/finding.rs), [status](src/status.rs), the
[check](src/check_id.rs) and [rule](src/rule_id.rs) identifiers and their [error](src/identifier_error.rs).
