import { describe, expect, it } from "@jest/globals";
import { TranslationFinding, TranslationVerifyResult } from "@xrf/ipc/types/xrf-translation";

import { IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

import { describeTranslationVerifyOutcome } from "./describe-verify-outcome";

function completed(overrides: Partial<TranslationVerifyResult> = {}): IJobOutcome<TranslationVerifyResult> {
  return {
    error: null,
    isCancelRequested: false,
    result: {
      checkedTranslationsCount: 2,
      duration: 1,
      languages: [],
      missingTranslationsCount: 0,
      outcome: "completed",
      problems: [],
      ...overrides,
    },
  };
}

describe("describeTranslationVerifyOutcome", () => {
  it("should call complete sources complete", () => {
    expect(describeTranslationVerifyOutcome("C:\\src", completed())).toEqual({
      details: "C:\\src",
      severity: ENotificationSeverity.SUCCESS,
      title: "Translations are complete",
    });
  });

  it("should warn about missing translations, which the game shows as their ids", () => {
    expect(describeTranslationVerifyOutcome("C:\\src", completed({ missingTranslationsCount: 3 }))).toEqual({
      details: "C:\\src",
      severity: ENotificationSeverity.WARNING,
      title: "Translations are incomplete: 3 missing",
    });
  });

  it("should report what the game would refuse or show wrong above anything missing", () => {
    const problem: TranslationFinding = {
      message: "'st_a' text in 'eng' holds 'é' (U+00E9)",
      rule: "translations.unencodable",
      subject: "st.json",
    };

    expect(
      describeTranslationVerifyOutcome("C:\\src", completed({ missingTranslationsCount: 3, problems: [problem] }))
    ).toEqual({
      details: "C:\\src",
      severity: ENotificationSeverity.ERROR,
      title: "Translations the game would refuse or show wrong: 1 problem(s)",
    });
  });
});
