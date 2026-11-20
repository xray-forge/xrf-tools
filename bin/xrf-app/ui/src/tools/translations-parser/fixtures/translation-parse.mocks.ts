import { CheckReport, Finding } from "@xrf/ipc/types/xrf-report";
import { TranslationParseCensus, TranslationParseResult } from "@xrf/ipc/types/xrf-translation";

/**
 * Creates an import census fixture that moved nothing.
 *
 * @param overrides - Counts to override.
 * @returns A census fixture.
 */
export function mockTranslationParseCensus(overrides: Partial<TranslationParseCensus> = {}): TranslationParseCensus {
  return {
    filesRead: 0,
    filesCreated: 0,
    filesUpdated: 0,
    filesUnchanged: 0,
    filesSkipped: 0,
    entriesRead: 0,
    entriesInserted: 0,
    entriesFilled: 0,
    entriesUnchanged: 0,
    entriesConflicted: 0,
    placeholdersAdded: 0,
    ...overrides,
  };
}

/**
 * Creates one report check fixture holding the findings given.
 *
 * @param id - The check's identity.
 * @param findings - What the check found.
 * @returns A check fixture.
 */
export function mockTranslationParseCheck(id: string, findings: Array<Finding> = []): CheckReport {
  return { id, duration: null, findings, status: findings.length ? "failed" : "passed" };
}

/**
 * Creates a completed, written import fixture with no findings.
 *
 * @param overrides - Fields to override.
 * @returns An import result fixture.
 */
export function mockTranslationParseResult(overrides: Partial<TranslationParseResult> = {}): TranslationParseResult {
  return {
    outcome: "completed",
    duration: 10,
    language: "eng",
    isDryRun: false,
    census: mockTranslationParseCensus(),
    report: { checks: [], status: "passed" },
    ...overrides,
  };
}
