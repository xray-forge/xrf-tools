import { ERenderTextureGroup } from "@xrf/ipc/types/xrf-renderer";

import { ILevelTextureProblem } from "@/core/level/lib/texture/level-texture-report";
import { IEditorProblem } from "@/core/shell/editor/EditorProblemsPanel";

import { ELevelProblemRule } from "./level-problem-rule";

/**
 * What the texture set has to say, as rows, the grass's apart from the surfaces'.
 *
 * @param textures - Every reference it could not answer for properly, with its reason.
 * @returns The rows, in the order the set read them.
 */
export function listTextureProblems(textures: ReadonlyArray<ILevelTextureProblem>): Array<IEditorProblem> {
  return textures.map((problem: ILevelTextureProblem) => ({
    message: problem.reason,
    rule: problem.group === ERenderTextureGroup.GRASS ? ELevelProblemRule.GRASS_TEXTURE : ELevelProblemRule.TEXTURE,
    subject: problem.reference,
  }));
}
