import { DEFAULT_LEVEL_SURFACE_OPTIONS, ILevelSurfaceOptions } from "@/core/level/lib/surface/level-surface-material";

/** Everything the level viewer's toolbar switches, which is more than what a surface is drawn with. */
export interface ILevelViewOptions extends ILevelSurfaceOptions {
  /**  Draws the ground plane the level sits on, in cells of a round number of metres, and the extent it claims over it. */
  isGridVisible: boolean;
  /** Draws the marker at the origin, which is the only thing that says which way `+x` and `+z` go. */
  isAxesVisible: boolean;
  /** Draws the sun in the sky, which is the only thing that says where the light is coming from. */
  isSunVisible: boolean;
}

export const DEFAULT_LEVEL_VIEW_OPTIONS: ILevelViewOptions = {
  ...DEFAULT_LEVEL_SURFACE_OPTIONS,
  isAxesVisible: true,
  isGridVisible: true,
  isSunVisible: true,
};
