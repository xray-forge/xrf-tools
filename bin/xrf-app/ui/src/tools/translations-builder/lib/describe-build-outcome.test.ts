import { describe, expect, it } from "@jest/globals";
import { TranslationBuildResult } from "@xrf/ipc/types/xrf-translation";

import { IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

import { describeTranslationBuildOutcome } from "./describe-build-outcome";

const OUTPUT: string = "C:\\work\\gamedata\\configs\\text";

function result(patch: Partial<TranslationBuildResult> = {}): TranslationBuildResult {
  return { outcome: "completed", duration: 10, sources: 12, files: 24, languages: [], ...patch };
}

function outcome(patch: Partial<IJobOutcome<TranslationBuildResult>>): IJobOutcome<TranslationBuildResult> {
  return { isCancelRequested: false, result: null, error: null, ...patch };
}

describe("describeTranslationBuildOutcome", () => {
  it("counts the string tables written and the sources they came from", () => {
    expect(describeTranslationBuildOutcome(OUTPUT, outcome({ result: result() }))).toEqual({
      details: OUTPUT,
      severity: ENotificationSeverity.SUCCESS,
      title: "Built 24 string table(s) from 12 source(s)",
    });
  });

  it("reports a stopped build as information, since running it again finishes the job", () => {
    const notice = describeTranslationBuildOutcome(
      OUTPUT,
      outcome({ result: result({ outcome: "cancelled", files: 1500, sources: 750 }) })
    );

    expect(notice.severity).toBe(ENotificationSeverity.INFO);
    expect(notice.title).toBe("Stopped building translations");
    expect(notice.details).toBe(
      [
        "Wrote 1,500 string table(s) from 750 source(s).",
        "The rest were not compiled; running it again finishes the job.",
        OUTPUT,
      ].join("\n")
    );
  });

  it("names the output and the reason when the build fails", () => {
    expect(
      describeTranslationBuildOutcome(OUTPUT, outcome({ error: new Error("Output directory is read only") }))
    ).toEqual({
      details: `${OUTPUT}\nOutput directory is read only`,
      severity: ENotificationSeverity.ERROR,
      title: "Could not build translations",
    });
  });

  it("reports a failure over a result the run also carried", () => {
    const notice = describeTranslationBuildOutcome(
      OUTPUT,
      outcome({ result: result(), error: new Error("Lost the backend") })
    );

    expect(notice.severity).toBe(ENotificationSeverity.ERROR);
  });

  it("counts nothing for a run that answered no result", () => {
    expect(describeTranslationBuildOutcome(OUTPUT, outcome({})).title).toBe("Built 0 string table(s) from 0 source(s)");
  });
});
