import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { listDrawableProblems } from "@/core/level/lib/problems/level-drawable-problems";
import { listSurfaceProblems } from "@/core/level/lib/problems/level-surface-problems";
import { listTextureProblems } from "@/core/level/lib/problems/level-texture-problems";
import { ILoadedSector } from "@/core/level/lib/sector/level-sector-set";
import { ILevelTextureProblem } from "@/core/level/lib/texture/level-texture-set";
import { IEditorProblem } from "@/core/shell/editor/EditorProblemsPanel";

/**
 * Everything the viewer could not draw as the level asked, as rows.
 *
 * @param textures - What the texture set has to say, which is a reference and a reason.
 * @param surfaces - The level's resolved shader table, in its own order.
 * @param sectors - The sectors currently resident, whose packs carry what they could not read.
 * @returns The rows, textures first, then surfaces, then drawables.
 */
export function listLevelProblems(
  textures: ReadonlyArray<ILevelTextureProblem>,
  surfaces: ReadonlyArray<XraySurfaceDescriptor>,
  sectors: ReadonlyMap<number, ILoadedSector>
): Array<IEditorProblem> {
  return [...listTextureProblems(textures), ...listSurfaceProblems(surfaces), ...listDrawableProblems(sectors)];
}
