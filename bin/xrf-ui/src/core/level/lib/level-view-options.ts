import { DEFAULT_LEVEL_SURFACE_OPTIONS, ILevelSurfaceOptions } from "@/core/level/lib/level-surface-material";

/** Everything the level viewer's toolbar switches, which is more than what a surface is drawn with. */
export interface ILevelViewOptions extends ILevelSurfaceOptions {
  /** Draws the ground plane the level sits on, in cells of a round number of metres. */
  isGridVisible: boolean;
  /** Draws the marker at the origin, which is the only thing that says which way `+x` and `+z` go. */
  isAxesVisible: boolean;
  /** Draws the box around the extent the level claims. */
  isBoundsVisible: boolean;
}

export const DEFAULT_LEVEL_VIEW_OPTIONS: ILevelViewOptions = {
  ...DEFAULT_LEVEL_SURFACE_OPTIONS,
  isAxesVisible: true,
  isBoundsVisible: true,
  isGridVisible: true,
};
