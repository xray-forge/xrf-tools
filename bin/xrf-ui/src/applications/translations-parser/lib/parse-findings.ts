import { Finding } from "@/core/ipc/types/xrf-report";
import { TranslationParseResult } from "@/core/ipc/types/xrf-translation";

/**
 * @param result - What an import run read.
 * @returns Every finding its report holds, check by check, in the order the run reported them.
 */
export function listTranslationParseFindings(result: TranslationParseResult): Array<Finding> {
  return result.report.checks.flatMap((check) => check.findings);
}
