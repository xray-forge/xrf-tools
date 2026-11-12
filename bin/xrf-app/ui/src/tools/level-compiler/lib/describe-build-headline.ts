import { LevelCompileReport, LevelCompileStepReport } from "@xrf/ipc/types/xrf-level-compiler";
import { LevelBuildReport } from "@xrf/ipc/types/xrf-rawdata";
import { Nullable } from "@xrf/types";

/**
 * The one sentence a build's result opens with: what was built where, and what the compile after it ran or that it
 * stopped.
 *
 * @param report - What the build wrote.
 * @param compile - What the compile ran, where one ran to its end.
 * @param isCompileStopped - Whether the run was cancelled after the build was written.
 * @returns The headline.
 */
export function describeBuildHeadline(
  report: LevelBuildReport,
  compile: Nullable<LevelCompileReport>,
  isCompileStopped: boolean
): string {
  const built: string = `Built ${report.source} into ${report.destination}`;

  if (isCompileStopped) {
    return `${built}; the compile was stopped, and what it wrote so far stays`;
  }

  if (compile && compile.steps.length > 0) {
    return `${built} and compiled it: ${compile.steps.map((step: LevelCompileStepReport) => step.step).join(", ")}`;
  }

  return built;
}
