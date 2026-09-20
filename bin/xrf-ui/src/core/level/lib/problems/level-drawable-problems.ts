import { EVisualSkipCause, SectorSkip } from "@/core/ipc/types/xrf-visual";
import { ELevelProblemRule } from "@/core/level/lib/problems/level-problem-rule";
import { ILoadedSector } from "@/core/level/lib/sector/level-sector-set";
import { IEditorProblem } from "@/core/shell/editor/EditorProblemsPanel";

/**
 * What each resident sector could not pack, which is geometry simply absent from the picture.
 *
 * @param sectors - The sectors currently resident, whose packs carry what they could not read.
 * @returns The rows, each naming the sector and the visual.
 */
export function listDrawableProblems(sectors: ReadonlyMap<number, ILoadedSector>): Array<IEditorProblem> {
  const problems: Array<IEditorProblem> = [];

  for (const [sector, loaded] of sectors) {
    for (const skip of loaded.views.skipped) {
      problems.push({
        message: `${describeCause(skip)}: ${skip.reason}`,
        rule: ELevelProblemRule.DRAWABLE,
        subject: `sector ${sector}, visual ${skip.drawable}`,
      });
    }
  }

  return problems;
}

/** Whether the reader does not model the form, or the form is broken. */
function describeCause(skip: SectorSkip): string {
  return skip.cause === EVisualSkipCause.UNSUPPORTED ? "Stored in a form the reader does not model" : "Malformed";
}
