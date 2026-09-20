import { DEFAULT_LEVEL_SURFACE_OPTIONS, ILevelSurfaceOptions } from "@/core/level/lib/level-surface-material";

/** Everything the level viewer's toolbar switches, which is more than what a surface is drawn with. */
export interface ILevelViewOptions extends ILevelSurfaceOptions {
  /** Draws the ground plane, the origin and the level's own extent, in the coordinates the level is written in. */
  isGridVisible: boolean;
}

export const DEFAULT_LEVEL_VIEW_OPTIONS: ILevelViewOptions = {
  ...DEFAULT_LEVEL_SURFACE_OPTIONS,
  isGridVisible: true,
};
