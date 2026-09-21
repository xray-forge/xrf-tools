import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { ILevelSectorSkip } from "@/core/level/lib/sector/level-sector-report";
import { ILevelTextureProblem } from "@/core/level/lib/texture/level-texture-report";
import { IEditorProblem } from "@/core/shell/editor/EditorProblemsPanel";

import { listDrawableProblems } from "./level-drawable-problems";
import { listSurfaceProblems } from "./level-surface-problems";
import { listTextureProblems } from "./level-texture-problems";

/**
 * Everything the viewer could not draw as the level asked, as rows.
 *
 * @param textures - What the texture set has to say, which is a reference and a reason.
 * @param surfaces - The level's resolved shader table, in its own order.
 * @param skipped - What the resident packs could not read.
 * @returns The rows, textures first, then surfaces, then drawables.
 */
export function listLevelProblems(
  textures: ReadonlyArray<ILevelTextureProblem>,
  surfaces: ReadonlyArray<XraySurfaceDescriptor>,
  skipped: ReadonlyArray<ILevelSectorSkip>
): Array<IEditorProblem> {
  return [...listTextureProblems(textures), ...listSurfaceProblems(surfaces), ...listDrawableProblems(skipped)];
}
