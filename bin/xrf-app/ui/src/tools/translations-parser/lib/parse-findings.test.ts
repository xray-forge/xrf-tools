import { describe, expect, it } from "@jest/globals";
import { Finding } from "@xrf/ipc/types/xrf-report";

import {
  mockTranslationParseCheck,
  mockTranslationParseResult,
} from "@/tools/translations-parser/fixtures/translation-parse.mocks";

import { listTranslationParseFindings } from "./parse-findings";

const MALFORMED: Finding = { ruleId: "translations.malformed", subject: "st_items.xml", message: "Unclosed tag" };
const DUPLICATE: Finding = { ruleId: "translations.duplicate", subject: "st_dialogs.xml", message: "'st_a' twice" };
const CONFLICT: Finding = { ruleId: "translations.conflict", subject: null, message: "'st_b' differs" };

describe("listTranslationParseFindings", () => {
  it("has nothing to list for a report whose checks found nothing", () => {
    const result = mockTranslationParseResult({
      report: { checks: [mockTranslationParseCheck("read"), mockTranslationParseCheck("merge")], status: "passed" },
    });

    expect(listTranslationParseFindings(result)).toEqual([]);
  });

  it("lists every check's findings, check by check, in the order the run reported them", () => {
    const result = mockTranslationParseResult({
      report: {
        checks: [
          mockTranslationParseCheck("read", [MALFORMED, DUPLICATE]),
          mockTranslationParseCheck("empty"),
          mockTranslationParseCheck("merge", [CONFLICT]),
        ],
        status: "failed",
      },
    });

    expect(listTranslationParseFindings(result)).toEqual([MALFORMED, DUPLICATE, CONFLICT]);
  });
});
