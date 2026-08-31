import { describe, expect, it } from "@jest/globals";

import { describeGamedataVerifyOutcome } from "@/applications/gamedata-verifier/lib/describe-gamedata-verify-outcome";
import { GamedataCheckSummary, GamedataVerifySummary } from "@/core/bindings/types/xrf-app";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

const ROOT: string = "C:\\Games\\Anomaly\\gamedata";

function check(patch: Partial<GamedataCheckSummary> = {}): GamedataCheckSummary {
  return { check: "ltx", status: "passed", summary: "", findings: 0, duration: 100, ...patch };
}

function summary(patch: Partial<GamedataVerifySummary> = {}): GamedataVerifySummary {
  return { outcome: "completed", status: "passed", checks: [check()], duration: 1000, ...patch };
}

function outcome(patch: Partial<IJobOutcome<GamedataVerifySummary>>): IJobOutcome<GamedataVerifySummary> {
  return { isCancelRequested: false, result: null, error: null, ...patch };
}

describe("describeGamedataVerifyOutcome", () => {
  it("passes a project where every check passed", () => {
    const notice: IJobNotice = describeGamedataVerifyOutcome(ROOT, outcome({ result: summary() }));

    expect(notice.severity).toBe(ENotificationSeverity.SUCCESS);
    expect(notice.title).toContain("passed");
  });

  it("names the checks that failed rather than only counting them", () => {
    const notice: IJobNotice = describeGamedataVerifyOutcome(
      ROOT,
      outcome({
        result: summary({
          status: "failed",
          checks: [check(), check({ check: "meshes", status: "failed", summary: "12 missing textures", findings: 12 })],
        }),
      })
    );

    expect(notice.severity).toBe(ENotificationSeverity.ERROR);
    expect(notice.details).toContain("meshes: 12 missing textures");
  });

  it("treats an incomplete run as a warning rather than a failure", () => {
    // A check that could not read everything it needed has not found a problem; it has found that it could not look.
    const notice: IJobNotice = describeGamedataVerifyOutcome(
      ROOT,
      outcome({
        result: summary({
          status: "incomplete",
          checks: [check({ status: "incomplete", summary: "two sources could not be opened" })],
        }),
      })
    );

    expect(notice.severity).toBe(ENotificationSeverity.WARNING);
  });

  it("does not count a skipped check as a failure", () => {
    const notice: IJobNotice = describeGamedataVerifyOutcome(
      ROOT,
      outcome({ result: summary({ checks: [check(), check({ check: "levels", status: "skipped" })] }) })
    );

    expect(notice.severity).toBe(ENotificationSeverity.SUCCESS);
  });

  it("never reports a stopped run as a verdict", () => {
    // The checks that ran are real answers; the ones that never started said nothing, and reporting that silence as a
    // pass is the one way a partial check can do harm.
    const notice: IJobNotice = describeGamedataVerifyOutcome(
      ROOT,
      outcome({ isCancelRequested: true, result: summary({ outcome: "cancelled" }) })
    );

    expect(notice.severity).toBe(ENotificationSeverity.INFO);
    expect(notice.title).toContain("Stopped");
    expect(notice.details).toContain("not run");
  });

  it("reports a failure with the reason", () => {
    const notice: IJobNotice = describeGamedataVerifyOutcome(ROOT, outcome({ error: new Error("root is unreadable") }));

    expect(notice.severity).toBe(ENotificationSeverity.ERROR);
    expect(notice.details).toContain("root is unreadable");
  });
});
