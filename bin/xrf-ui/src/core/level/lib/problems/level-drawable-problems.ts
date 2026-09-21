import { EVisualSkipCause, SectorSkip } from "@/core/ipc/types/xrf-visual";
import { ELevelProblemRule } from "@/core/level/lib/problems/level-problem-rule";
import { ILevelSectorSkip } from "@/core/level/lib/sector/level-sector-report";
import { IEditorProblem } from "@/core/shell/editor/EditorProblemsPanel";

/**
 * What each resident sector could not pack, which is geometry simply absent from the picture.
 *
 * @param skipped - What the resident packs could not read.
 * @returns The rows, each naming the sector and the visual.
 */
export function listDrawableProblems(skipped: ReadonlyArray<ILevelSectorSkip>): Array<IEditorProblem> {
  return skipped.map(({ sector, skip }) => ({
    message: `${describeCause(skip)}: ${skip.reason}`,
    rule: ELevelProblemRule.DRAWABLE,
    subject: `sector ${sector}, visual ${skip.drawable}`,
  }));
}

/** Whether the reader does not model the form, or the form is broken. */
function describeCause(skip: SectorSkip): string {
  return skip.cause === EVisualSkipCause.UNSUPPORTED ? "Stored in a form the reader does not model" : "Malformed";
}
