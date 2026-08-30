import { describe, expect, it } from "@jest/globals";

import { describeFormatOutcome } from "@/applications/configs-formatter/lib/describe-format-outcome";
import { LtxProjectFormatResult } from "@/core/bindings/types/xrf-ltx";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

const DIRECTORY: string = "C:\\work\\gamedata\\configs";

function result(patch: Partial<LtxProjectFormatResult> = {}): LtxProjectFormatResult {
  return { outcome: "completed", totalFiles: 2665, invalidFiles: 0, toFormat: [], ...patch } as LtxProjectFormatResult;
}

function outcome(patch: Partial<IJobOutcome<LtxProjectFormatResult>>): IJobOutcome<LtxProjectFormatResult> {
  return { isCancelRequested: false, result: null, error: null, ...patch };
}

describe("describeFormatOutcome", () => {
  describe("checking", () => {
    it("never claims to have formatted anything", () => {
      // The regression this file exists for. A check writes nothing, and reporting its count as files "formatted"
      // told the user their project had been rewritten when it had only been read.
      const notice: IJobNotice = describeFormatOutcome(
        DIRECTORY,
        true,
        outcome({ result: result({ invalidFiles: 2481 }) })
      );

      expect(notice.title).not.toContain("Formatted");
      expect(notice.title).toContain("invalid formatting");
    });

    it("treats badly formatted files as a failed check", () => {
      // The same verdict the command line reaches with the same question: `ltx format --check` exits 3.
      const notice: IJobNotice = describeFormatOutcome(
        DIRECTORY,
        true,
        outcome({ result: result({ invalidFiles: 2481 }) })
      );

      expect(notice.severity).toBe(ENotificationSeverity.ERROR);
    });

    it("passes a project that is already formatted", () => {
      const notice: IJobNotice = describeFormatOutcome(DIRECTORY, true, outcome({ result: result() }));

      expect(notice.severity).toBe(ENotificationSeverity.SUCCESS);
      expect(notice.title).toContain("correct format");
    });

    it("says it read rather than rewrote when stopped", () => {
      const notice: IJobNotice = describeFormatOutcome(
        DIRECTORY,
        true,
        outcome({ isCancelRequested: true, result: result({ outcome: "cancelled", invalidFiles: 12 }) })
      );

      expect(notice.title).toBe("Stopped checking formatting");
      expect(notice.details).not.toContain("rewritten");
    });

    it("names checking rather than formatting when it fails", () => {
      const notice: IJobNotice = describeFormatOutcome(DIRECTORY, true, outcome({ error: new Error("unreadable") }));

      expect(notice.title).toBe("Could not check formatting");
    });
  });

  describe("formatting", () => {
    it("reports what it rewrote as a success", () => {
      const notice: IJobNotice = describeFormatOutcome(
        DIRECTORY,
        false,
        outcome({ result: result({ invalidFiles: 2481 }) })
      );

      expect(notice.severity).toBe(ENotificationSeverity.SUCCESS);
      expect(notice.title).toContain("Formatted");
    });

    it("says what it left alone when stopped", () => {
      // A stopped rewrite is recoverable rather than damaging: each file goes through a staged replace, so the rest
      // are simply untouched and running it again finishes the job.
      const notice: IJobNotice = describeFormatOutcome(
        DIRECTORY,
        false,
        outcome({ isCancelRequested: true, result: result({ outcome: "cancelled", invalidFiles: 40 }) })
      );

      expect(notice.severity).toBe(ENotificationSeverity.INFO);
      expect(notice.title).toBe("Stopped formatting configs");
      expect(notice.details).toContain("running it again");
    });

    it("names formatting rather than checking when it fails", () => {
      const notice: IJobNotice = describeFormatOutcome(DIRECTORY, false, outcome({ error: new Error("unreadable") }));

      expect(notice.title).toBe("Could not format configs");
    });
  });
});
