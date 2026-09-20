import { ELevelProblemRule } from "@/core/level/lib/problems/level-problem-rule";
import { ILevelTextureProblem } from "@/core/level/lib/texture/level-texture-set";
import { IEditorProblem } from "@/core/shell/editor/EditorProblemsPanel";

/**
 * What the texture set has to say, as rows.
 *
 * @param textures - Every reference it could not answer for properly, with its reason.
 * @returns The rows, in the order the set read them.
 */
export function listTextureProblems(textures: ReadonlyArray<ILevelTextureProblem>): Array<IEditorProblem> {
  return textures.map((problem: ILevelTextureProblem) => ({
    message: problem.reason,
    rule: ELevelProblemRule.TEXTURE,
    subject: problem.reference,
  }));
}
