import { LevelSpawnModelFailure } from "@/core/ipc/types/xrf-app";
import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { ILevelSectorSkip } from "@/core/level/lib/sector/level-sector-report";
import { ILevelTextureProblem } from "@/core/level/lib/texture/level-texture-report";
import { IEditorProblem } from "@/core/shell/editor/EditorProblemsPanel";

import { listDrawableProblems } from "./level-drawable-problems";
import { listSpawnProblems } from "./level-spawn-problems";
import { listSurfaceProblems } from "./level-surface-problems";
import { listTextureProblems } from "./level-texture-problems";

/** Everything a level's problems are listed from. */
export interface ILevelProblemSources {
  /** What the texture set has to say, which is a reference and a reason. */
  textures: ReadonlyArray<ILevelTextureProblem>;
  /** The level's resolved shader table, in its own order. */
  surfaces: ReadonlyArray<XraySurfaceDescriptor>;
  /** What the resident packs could not read. */
  skipped: ReadonlyArray<ILevelSectorSkip>;
  /** The spawned visuals that could not be read. */
  spawn: ReadonlyArray<LevelSpawnModelFailure>;
}

/**
 * Everything the viewer could not draw as the level asked, as rows.
 *
 * @param sources - What the problems are listed from.
 * @returns The rows, textures first, then surfaces, drawables and spawned visuals.
 */
export function listLevelProblems(sources: ILevelProblemSources): Array<IEditorProblem> {
  return [
    ...listTextureProblems(sources.textures),
    ...listSurfaceProblems(sources.surfaces),
    ...listDrawableProblems(sources.skipped),
    ...listSpawnProblems(sources.spawn),
  ];
}
