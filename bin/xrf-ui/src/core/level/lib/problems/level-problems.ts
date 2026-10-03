import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { RenderLoadFailure, RenderSectorSkip } from "@/core/ipc/types/xrf-renderer";
import { ILevelSpawnReport } from "@/core/level/lib/spawn/level-spawn-report";
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
  skipped: ReadonlyArray<RenderSectorSkip>;
  /** The sectors that could not be read at all, by their index. */
  sectors: ReadonlyArray<RenderLoadFailure>;
  /** What the spawn's listing came to. */
  spawn: ILevelSpawnReport;
  /** The visuals spawned objects name that could not be read. */
  models: ReadonlyArray<RenderLoadFailure>;
}

/**
 * Everything the viewer could not draw as the level asked, as rows.
 *
 * @param sources - What the problems are listed from.
 * @returns The rows, textures first, then surfaces, sectors and their drawables, and spawned visuals.
 */
export function listLevelProblems(sources: ILevelProblemSources): Array<IEditorProblem> {
  return [
    ...listTextureProblems(sources.textures),
    ...listSurfaceProblems(sources.surfaces),
    ...listDrawableProblems(sources.skipped, sources.sectors),
    ...listSpawnProblems(sources.spawn, sources.models),
  ];
}
