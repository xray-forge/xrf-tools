# xrf-report

Immutable, serializable results for command checks and findings.

## Build a report

```rust
use xrf_report::{CheckId, CheckReport, Finding, Report, RuleId, Status};

fn main() -> xrf_error::XrfResult {
  let finding = Finding::new(
    RuleId::new("textures.missing")?,
    Some("textures/example.dds".to_owned()),
    "Referenced texture was not found",
  );

  let check = CheckReport::new(CheckId::new("textures")?, Status::Failed, None, vec![finding]);
  let report = Report::new(vec![check]);

  assert_eq!(report.status(), Status::Failed);
  assert_eq!(report.checks()[0].findings().len(), 1);

  Ok(())
}
```

## Concepts

Create a `Finding` with a validated `RuleId`, optional subject, and message. Collect findings into a `CheckReport`
with a `CheckId`, explicit `Status`, and optional duration, then pass the checks to `Report::new`.

`CheckReport::new` sorts findings deterministically. `Report::new` preserves check order and aggregates their statuses;
it does not infer a check's status from its findings. Durations serialize as whole milliseconds, or remain absent
when unmeasured.

`CheckId` names an operation's check; `RuleId` identifies one rule within it. Both reject an empty string and serialize
as strings. They do not impose a dotted naming scheme; stable naming is the producer's responsibility.

The optional finding subject identifies the affected asset or value. It is report text, not a validated filesystem
path. Keep absence as `None` when a finding concerns the check as a whole.

## Status and serialization

Statuses aggregate in this order, strongest first: `Error`, `Failed`, `Incomplete`, `Passed`, `Skipped`. An empty
report is `Skipped`. Findings alone do not determine which status is appropriate; supply the check's verdict explicitly.

The report and its checks serialize with camelCase field names; statuses use lowercase strings. Construction fixes
the order before serialization, so output does not depend on worker completion order.

## Checks and related crates

This crate stores the final result. Use [xrf-output](../xrf-output/README.md) for live messages and
[xrf-job](../xrf-job/README.md) for progress.

Run `cargo test --locked -p xrf-report`. Report construction does not log, write a JSON file, or choose an exit code;
those decisions belong to the consuming command.

See the [report example and tests](src/report.rs), [finding](src/finding.rs), and [identifier rules](src/identifier.rs).
