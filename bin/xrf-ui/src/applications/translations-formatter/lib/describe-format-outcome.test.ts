import { describe, expect, it } from "@jest/globals";

import { describeFormatOutcome } from "@/applications/translations-formatter/lib/describe-format-outcome";
import { ProjectFormatResult } from "@/core/bindings/types/xrf-translation";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

const DIRECTORY: string = "C:\\work\\xrf-engine\\src\\engine\\translations";

function result(patch: Partial<ProjectFormatResult> = {}): ProjectFormatResult {
  return { outcome: "completed", totalFiles: 34, invalidFiles: 0, toFormat: [], ...patch } as ProjectFormatResult;
}

function outcome(patch: Partial<IJobOutcome<ProjectFormatResult>>): IJobOutcome<ProjectFormatResult> {
  return { isCancelRequested: false, result: null, error: null, ...patch };
}

describe("describeFormatOutcome", () => {
  describe("checking", () => {
    it("never claims to have formatted anything", () => {
      // A check writes nothing, and reporting its count as sources "formatted" would tell the user their project had
      // been rewritten when it had only been read.
      const notice: IJobNotice = describeFormatOutcome(
        DIRECTORY,
        true,
        outcome({ result: result({ invalidFiles: 23 }) })
      );

      expect(notice.title).not.toContain("Formatted");
      expect(notice.title).toContain("not formatted");
    });

    it("treats unformatted sources as a failed check", () => {
      // The same verdict the command line reaches with the same question: `translation format --check` exits 3.
      const notice: IJobNotice = describeFormatOutcome(
        DIRECTORY,
        true,
        outcome({ result: result({ invalidFiles: 23 }) })
      );

      expect(notice.severity).toBe(ENotificationSeverity.ERROR);
    });

    it("passes a tree that is already canonical", () => {
      const notice: IJobNotice = describeFormatOutcome(DIRECTORY, true, outcome({ result: result() }));

      expect(notice.severity).toBe(ENotificationSeverity.SUCCESS);
      expect(notice.title).toBe("All translation sources are in correct format");
    });
  });

  describe("formatting", () => {
    it("reports the rewrite as work done rather than as a failure", () => {
      const notice: IJobNotice = describeFormatOutcome(
        DIRECTORY,
        false,
        outcome({ result: result({ invalidFiles: 23 }) })
      );

      expect(notice.severity).toBe(ENotificationSeverity.SUCCESS);
      expect(notice.title).toBe("Formatted 23 of 34 translation source(s)");
    });
  });

  describe("stopping", () => {
    it("says what a stopped rewrite left behind", () => {
      // Each source is a staged replace, so the answer is "some done, rest untouched" and never "one half-written".
      const notice: IJobNotice = describeFormatOutcome(
        DIRECTORY,
        false,
        outcome({ result: result({ outcome: "cancelled", invalidFiles: 4, totalFiles: 9 }) })
      );

      expect(notice.severity).toBe(ENotificationSeverity.INFO);
      expect(notice.title).toBe("Stopped formatting translation sources");
      expect(notice.details).toContain("running it again finishes the job");
    });

    it("does not say a stopped check left anything behind", () => {
      const notice: IJobNotice = describeFormatOutcome(
        DIRECTORY,
        true,
        outcome({ result: result({ outcome: "cancelled", invalidFiles: 4, totalFiles: 9 }) })
      );

      expect(notice.title).toBe("Stopped checking translation formatting");
      expect(notice.details).toContain("was not read");
    });
  });

  describe("failing", () => {
    it("names the mode that failed", () => {
      const error: Error = new Error("Close the open translations project first");

      expect(describeFormatOutcome(DIRECTORY, false, outcome({ error })).title).toBe(
        "Could not format translation sources"
      );
      expect(describeFormatOutcome(DIRECTORY, true, outcome({ error })).title).toBe(
        "Could not check translation formatting"
      );
      expect(describeFormatOutcome(DIRECTORY, false, outcome({ error })).severity).toBe(ENotificationSeverity.ERROR);
    });
  });
});
