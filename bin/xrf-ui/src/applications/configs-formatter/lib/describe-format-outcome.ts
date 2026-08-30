import { LtxProjectFormatResult } from "@/core/bindings/types/xrf-ltx";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

/**
 * What the notification centre says when a configs formatting run ends.
 *
 * One describer for both modes, because one form runs both and they answer in the same shape. Every branch asks
 * `isCheck` first: the two report the same numbers and mean opposite things by them, and a check that announced files
 * as "formatted" would be claiming to have written to a project it never touched.
 *
 * A check that found badly formatted files is an error rather than a warning, matching what the command line does with
 * the same question — `ltx format --check` exits 3, a failed check. The rewrite that fixes them is a success.
 *
 * A stopped run is information either way. Each file is rewritten through a staged replace, so what a stopped rewrite
 * leaves behind is some files formatted and the rest untouched — a state running it again resolves, and never a file
 * half-written.
 *
 * @param directory - Configs directory the run was pointed at.
 * @param isCheck - Whether the run reported the formatting rather than repairing it.
 * @param outcome - How the run ended.
 * @returns What to record about it.
 */
export function describeFormatOutcome(
  directory: string,
  isCheck: boolean,
  outcome: IJobOutcome<LtxProjectFormatResult>
): IJobNotice {
  const { result, error } = outcome;

  if (error) {
    return {
      details: [directory, error.message].join("\n"),
      severity: ENotificationSeverity.ERROR,
      title: isCheck ? "Could not check formatting" : "Could not format configs",
    };
  }

  if (result?.outcome === "cancelled") {
    return {
      details: [
        isCheck
          ? `Read ${result.totalFiles.toLocaleString()} file(s), ${result.invalidFiles} badly formatted so far.`
          : `Read ${result.totalFiles.toLocaleString()} file(s), of which ${result.invalidFiles} were rewritten.`,
        isCheck
          ? "The rest of the project was not read."
          : "The rest were left as they were; running it again finishes the job.",
        directory,
      ].join("\n"),
      severity: ENotificationSeverity.INFO,
      title: isCheck ? "Stopped checking formatting" : "Stopped formatting configs",
    };
  }

  const invalid: number = result?.invalidFiles ?? 0;
  const total: number = result?.totalFiles ?? 0;

  if (isCheck) {
    return {
      details: directory,
      severity: invalid ? ENotificationSeverity.ERROR : ENotificationSeverity.SUCCESS,
      title: invalid
        ? `${invalid.toLocaleString()} of ${total.toLocaleString()} config file(s) have invalid formatting`
        : "All config files are in correct format",
    };
  }

  return {
    details: directory,
    severity: ENotificationSeverity.SUCCESS,
    title: `Formatted ${invalid.toLocaleString()} of ${total.toLocaleString()} config file(s)`,
  };
}
