import { RenderTextureReport } from "@xrf/ipc/types/xrf-renderer";
import { WorldLevelProblems, WorldSurfaceGeometry } from "@xrf/ipc/types/xrf-world";

import { ILevelSurfaceGeometry, toLevelSurfaceGeometry } from "@/core/level/lib/surface/level-surface-geometry";
import {
  EMPTY_LEVEL_TEXTURE_REPORT,
  ILevelTextureReport,
  toLevelTextureReport,
} from "@/core/level/lib/texture/level-texture-report";

/** What the renderer says of the open level once it is resident whole. */
export interface ILevelResident {
  /** What each shader table entry draws across the level, keyed by shader id. */
  surfaces: ReadonlyMap<number, ILevelSurfaceGeometry>;
  /** What the level's textures came to. */
  textures: ILevelTextureReport;
  /** What the level could not draw the way it asked. */
  problems: WorldLevelProblems;
}

/** Nothing measured, uploaded or failed, which is what a level not yet resident reports. */
export const EMPTY_LEVEL_RESIDENT: ILevelResident = {
  problems: { models: [], sectors: [], skipped: [] },
  surfaces: new Map(),
  textures: EMPTY_LEVEL_TEXTURE_REPORT,
};

/**
 * @param measured - What a native viewport counted each shader table entry of its level drawing.
 * @param textures - What it said became of each texture the level samples.
 * @param problems - What it could not draw the way the level asked.
 * @returns The same, as the level's panels read it.
 */
export function toLevelResident(
  measured: Array<WorldSurfaceGeometry>,
  textures: Array<RenderTextureReport>,
  problems: WorldLevelProblems
): ILevelResident {
  return { problems, surfaces: toLevelSurfaceGeometry(measured), textures: toLevelTextureReport(textures) };
}
