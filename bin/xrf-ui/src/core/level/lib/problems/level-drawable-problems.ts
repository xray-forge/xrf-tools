import { RenderLoadFailure } from "@/core/ipc/types/xrf-renderer";
import { EVisualSkipCause, SectorSkip } from "@/core/ipc/types/xrf-visual";
import { WorldSectorSkip } from "@/core/ipc/types/xrf-world";
import { ELevelProblemRule } from "@/core/level/lib/problems/level-problem-rule";
import { IEditorProblem } from "@/core/shell/editor/EditorProblemsPanel";

/**
 * What the sectors could not pack, which is geometry simply absent from the picture: a sector that could not be read
 * at all, then each drawable of one that was.
 *
 * @param skipped - What the resident packs could not read.
 * @param sectors - The sectors that could not be read, by their index.
 * @returns The rows, each naming the sector, and the visual where one was left out.
 */
export function listDrawableProblems(
  skipped: ReadonlyArray<WorldSectorSkip>,
  sectors: ReadonlyArray<RenderLoadFailure> = []
): Array<IEditorProblem> {
  return [
    ...sectors.map(({ name, reason }: RenderLoadFailure) => ({
      message: `Nothing of it is drawn: ${reason}`,
      rule: ELevelProblemRule.DRAWABLE,
      subject: `sector ${name}`,
    })),
    ...skipped.map(({ sector, skip }: WorldSectorSkip) => ({
      message: `${describeCause(skip)}: ${skip.reason}`,
      rule: ELevelProblemRule.DRAWABLE,
      subject: `sector ${sector}, visual ${skip.drawable}`,
    })),
  ];
}

/** Whether the reader does not model the form, or the form is broken. */
function describeCause(skip: SectorSkip): string {
  return skip.cause === EVisualSkipCause.UNSUPPORTED ? "Stored in a form the reader does not model" : "Malformed";
}
