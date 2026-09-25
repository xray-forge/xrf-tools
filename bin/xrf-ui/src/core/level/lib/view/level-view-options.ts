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
  /** Lists what each pass cost on the GPU in the frame readout, under what the frame cost as a whole. */
  isAdvancedStatsVisible: boolean;
  /** Draws the noon fog, which closes the level in at 350 metres as the game does. */
  isFogged: boolean;
  /** Draws a distant clump of trees as its impostor, as the game does, rather than every tree at every distance. */
  isImpostors: boolean;
  /** Casts the sun's shadows, while the settings draw them. */
  isShadowed: boolean;
  /** Darkens creases and corners by the screen's ambient occlusion, while the settings draw it. */
  isOccluded: boolean;
  /** Smooths the frame's edges, while the settings smooth them. */
  isAntialiased: boolean;
  /** Sways the trees and the grass in the wind, as the game does. */
  isWindy: boolean;
  /** Draws the grass, while the settings draw it. */
  isGrassy: boolean;
  /** Lights the level with its lamps, while the settings draw lights. */
  isLightsOn: boolean;
}

export const DEFAULT_LEVEL_VIEW_OPTIONS: ILevelViewOptions = {
  ...DEFAULT_LEVEL_SURFACE_OPTIONS,
  isAntialiased: true,
  isAxesVisible: false,
  isFogged: true,
  isGrassy: true,
  isGridVisible: false,
  isAdvancedStatsVisible: false,
  isImpostors: true,
  isLightsOn: true,
  isOccluded: true,
  isShadowed: true,
  isWindy: true,
  isStatsVisible: true,
  isSunVisible: true,
};
