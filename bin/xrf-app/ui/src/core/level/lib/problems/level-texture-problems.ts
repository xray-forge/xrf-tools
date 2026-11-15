import { ERenderTextureGroup } from "@xrf/ipc/types/xrf-renderer";
import { IEditorProblem } from "@xrf/ui/editor/problems";

import { ELevelTextureProblemRule } from "@/core/level/lib/problems/level-texture-problem-rule";
import { ILevelTextureProblem } from "@/core/level/lib/texture/level-texture-report";

/**
 * What the texture set has to say, as rows, the grass's apart from the surfaces'.
 *
 * @param textures - Every reference it could not answer for properly, with its reason.
 * @returns The rows, in the order the set read them.
 */
export function listTextureProblems(textures: ReadonlyArray<ILevelTextureProblem>): Array<IEditorProblem> {
  return textures.map((problem: ILevelTextureProblem) => ({
    message: problem.reason,
    rule:
      problem.group === ERenderTextureGroup.GRASS
        ? ELevelTextureProblemRule.GRASS_TEXTURE
        : ELevelTextureProblemRule.TEXTURE,
    subject: problem.reference,
  }));
}
