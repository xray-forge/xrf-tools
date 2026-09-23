import { DEFAULT_LEVEL_SURFACE_OPTIONS, ILevelSurfaceOptions } from "@/core/level/lib/surface/level-surface-options";

/** Everything the level viewer's toolbar switches, which is more than what a surface is drawn with. */
export interface ILevelViewOptions extends ILevelSurfaceOptions {
  /**  Draws the ground plane the level sits on, in cells of a round number of metres, and the extent it claims over it. */
  isGridVisible: boolean;
  /** Draws the axis marker at the level's own origin, which is the only thing that says which way `+x` and `+z` go. */
  isAxesVisible: boolean;
  /** Draws the sun in the sky, which is the only thing that says where the light is coming from. */
  isSunVisible: boolean;
  /** Draws the two readouts over the viewport: what the frame cost, and where the camera stands. */
  isStatsVisible: boolean;
  /** Draws the noon fog, which closes the level in at 350 metres as the game does. */
  isFogged: boolean;
}

export const DEFAULT_LEVEL_VIEW_OPTIONS: ILevelViewOptions = {
  ...DEFAULT_LEVEL_SURFACE_OPTIONS,
  isAxesVisible: false,
  isFogged: true,
  isGridVisible: true,
  isStatsVisible: true,
  isSunVisible: true,
};
