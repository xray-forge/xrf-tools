import { describe, expect, it } from "@jest/globals";
import { TranslationParseResult } from "@xrf/ipc/types/xrf-translation";

import { IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";
import {
  mockTranslationParseCensus,
  mockTranslationParseCheck,
  mockTranslationParseResult,
} from "@/tools/translations-parser/fixtures/translation-parse.mocks";

import { describeTranslationParseOutcome } from "./describe-parse-outcome";

const OUTPUT: string = "C:\\work\\xrf-engine\\src\\engine\\translations";

function outcome(patch: Partial<IJobOutcome<TranslationParseResult>>): IJobOutcome<TranslationParseResult> {
  return { isCancelRequested: false, result: null, error: null, ...patch };
}

describe("describeTranslationParseOutcome", () => {
  it("counts the tables read by an import that found nothing to object to", () => {
    const result = mockTranslationParseResult({ census: mockTranslationParseCensus({ filesRead: 7 }) });

    expect(describeTranslationParseOutcome(OUTPUT, outcome({ result }))).toEqual({
      details: OUTPUT,
      severity: ENotificationSeverity.SUCCESS,
      title: "Imported 7 table(s)",
    });
  });

  it("warns about an import that wrote its sources but reported findings", () => {
    const result = mockTranslationParseResult({
      census: mockTranslationParseCensus({ filesRead: 7 }),
      report: {
        checks: [
          mockTranslationParseCheck("read", [
            { ruleId: "translations.malformed", subject: "st_items.xml", message: "Unclosed tag" },
          ]),
          mockTranslationParseCheck("merge", [
            { ruleId: "translations.conflict", subject: "st_dialogs.xml", message: "'st_a' differs" },
          ]),
        ],
        status: "failed",
      },
    });

    expect(describeTranslationParseOutcome(OUTPUT, outcome({ result }))).toEqual({
      details: OUTPUT,
      severity: ENotificationSeverity.WARNING,
      title: "Imported 7 table(s), 2 finding(s)",
    });
  });

  it("says what a preview would change rather than claiming it imported anything", () => {
    const result = mockTranslationParseResult({
      isDryRun: true,
      census: mockTranslationParseCensus({ filesRead: 9, filesCreated: 2, filesUpdated: 3, filesUnchanged: 4 }),
    });

    expect(describeTranslationParseOutcome(OUTPUT, outcome({ result }))).toEqual({
      details: OUTPUT,
      severity: ENotificationSeverity.INFO,
      title: "Import would change 5 file(s)",
    });
  });

  it("warns about a preview that reported findings, as its result view does", () => {
    const result = mockTranslationParseResult({
      isDryRun: true,
      census: mockTranslationParseCensus({ filesCreated: 1_200, filesUpdated: 300 }),
      report: {
        checks: [
          mockTranslationParseCheck("read", [
            { ruleId: "translations.malformed", subject: "st_items.xml", message: "Unclosed tag" },
          ]),
        ],
        status: "failed",
      },
    });

    expect(describeTranslationParseOutcome(OUTPUT, outcome({ result }))).toEqual({
      details: OUTPUT,
      severity: ENotificationSeverity.WARNING,
      title: "Import would change 1,500 file(s), 1 finding(s)",
    });
  });

  it("reports a stopped import as information, since what was written stays complete", () => {
    const result = mockTranslationParseResult({
      outcome: "cancelled",
      census: mockTranslationParseCensus({ filesRead: 1200 }),
    });
    const notice = describeTranslationParseOutcome(OUTPUT, outcome({ result }));

    expect(notice.severity).toBe(ENotificationSeverity.INFO);
    expect(notice.title).toBe("Stopped importing translations");
    expect(notice.details).toBe(
      [
        "Read 1,200 table(s) before stopping.",
        "What was written was left in place; running it again finishes the job.",
        OUTPUT,
      ].join("\n")
    );
  });

  it("names the output and the reason when the import fails", () => {
    expect(describeTranslationParseOutcome(OUTPUT, outcome({ error: new Error("No string tables found") }))).toEqual({
      details: `${OUTPUT}\nNo string tables found`,
      severity: ENotificationSeverity.ERROR,
      title: "Could not import translations",
    });
  });

  it("counts nothing for a run that answered no result", () => {
    expect(describeTranslationParseOutcome(OUTPUT, outcome({}))).toEqual({
      details: OUTPUT,
      severity: ENotificationSeverity.SUCCESS,
      title: "Imported 0 table(s)",
    });
  });
});
