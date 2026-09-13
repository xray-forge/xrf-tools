import { BuildInfo } from "@/core/bindings/types/xrf-build-info";
import { getCommitUrl, getWorkflowRunUrl } from "@/core/configs";

import { IAboutRow, statedRows } from "./about-row";

/**
 * @param build - Build the backend reported.
 * @returns Every row worth stating about it, in the order they read.
 */
export function describeBuild(build: BuildInfo): Array<IAboutRow> {
  return statedRows([
    ["Version", `${build.version} (${build.kind})`],
    [
      "Commit",
      build.commit ? `${build.commit.slice(0, 7)}${build.isDirty ? " (dirty)" : ""}` : null,
      build.commit ? getCommitUrl(build.commit) : undefined,
    ],
    ["Branch", build.reference],
    ["Built", build.builtAt],
    ["Target", build.target],
    ["Compiler", build.rustc],
    ["Optimization", build.optimization],
    ["Workflow", build.runId, build.runId ? getWorkflowRunUrl(build.runId) : undefined],
  ]);
}
