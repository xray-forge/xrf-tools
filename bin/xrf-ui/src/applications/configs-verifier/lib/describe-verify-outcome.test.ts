import { describe, expect, it } from "@jest/globals";

import { describeVerifyOutcome } from "@/applications/configs-verifier/lib/describe-verify-outcome";
import { LtxProjectVerifyResult } from "@/core/bindings/types/xrf-ltx";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

const DIRECTORY: string = "C:\\work\\gamedata\\configs";

function result(patch: Partial<LtxProjectVerifyResult> = {}): LtxProjectVerifyResult {
  return { outcome: "completed", totalFiles: 2665, errors: [], ...patch } as LtxProjectVerifyResult;
}

function outcome(patch: Partial<IJobOutcome<LtxProjectVerifyResult>>): IJobOutcome<LtxProjectVerifyResult> {
  return { isCancelRequested: false, result: null, error: null, ...patch };
}

describe("describeVerifyOutcome", () => {
  it("passes a project with no findings", () => {
    const notice: IJobNotice = describeVerifyOutcome(DIRECTORY, outcome({ result: result() }));

    expect(notice.severity).toBe(ENotificationSeverity.SUCCESS);
    expect(notice.title).toContain("passed");
  });

  it("reports findings as a failed verdict", () => {
    const notice: IJobNotice = describeVerifyOutcome(
      DIRECTORY,
      outcome({ result: result({ errors: ["a", "b"] as unknown as LtxProjectVerifyResult["errors"] }) })
    );

    expect(notice.severity).toBe(ENotificationSeverity.WARNING);
    expect(notice.title).toContain("2 problem(s)");
  });

  it("never reports a stopped run as passing", () => {
    // The one way a partial check can do harm: it read part of the project, so its silence about the rest is not a
    // verdict. Saying "passed" here would be claiming a project was checked that never was.
    const notice: IJobNotice = describeVerifyOutcome(
      DIRECTORY,
      outcome({ isCancelRequested: true, result: result({ outcome: "cancelled", totalFiles: 400 }) })
    );

    expect(notice.severity).toBe(ENotificationSeverity.INFO);
    expect(notice.title).toContain("Stopped");
    expect(notice.details).toContain("not read");
  });

  it("reports a failure with the reason", () => {
    const notice: IJobNotice = describeVerifyOutcome(DIRECTORY, outcome({ error: new Error("unreadable root") }));

    expect(notice.severity).toBe(ENotificationSeverity.ERROR);
    expect(notice.details).toContain("unreadable root");
  });
});
