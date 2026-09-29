import { DEFAULT_LEVEL_SURFACE_OPTIONS, ILevelSurfaceOptions } from "@/core/level/lib/surface/level-surface-options";

/** Everything the level viewer's toolbar switches, which is more than what a surface is drawn with. */
export interface ILevelViewOptions extends ILevelSurfaceOptions {
  /** Draws the ground plane the level sits on, in cells of a round number of metres, and the extent it claims over it. */
  isGridVisible: boolean;
  /** Draws the axis marker at the level's own origin, which is the only thing that says which way `+x` and `+z` go. */
  isAxesVisible: boolean;
  /** Draws the sun in the sky, which is the only thing that says where the light is coming from. */
  isSunVisible: boolean;
  /** Draws the two readouts over the viewport: what the frame cost, and where the camera stands. */
  isStatsVisible: boolean;
  /** Draws the fog, which closes the level in at its distance. */
  isFogged: boolean;
  /** Draws the sky cube, rather than the backdrop behind the level. */
  isSkyVisible: boolean;
  /** Draws the clouds over the sky. */
  isClouded: boolean;
  /** Rains where the weather rains. */
  isRainy: boolean;
  /** Draws a distant clump of trees as its impostor, as the game does, rather than every tree at every distance. */
  isImpostors: boolean;
  /** Casts the sun's shadows, while the settings draw them. */
  isShadowed: boolean;
  /** Darkens creases and corners by the screen's ambient occlusion, while the settings draw it. */
  isOccluded: boolean;
  /** Culls the static draws the depth hides, while the settings cull them. */
  isOcclusionCulled: boolean;
  /** Smooths the frame's edges, while the settings smooth them. */
  isAntialiased: boolean;
  /** Sways the trees and the grass in the wind, as the game does. */
  isWindy: boolean;
  /** Draws the grass, while the settings draw it. */
  isGrassy: boolean;
  /** Lights the level with its lamps, while the settings draw lights. */
  isLamplit: boolean;
  /** Draws the water, while the settings draw it. */
  isWaterVisible: boolean;
}

/** The baked hemisphere occlusion applied whole, as the engine applies it. */
export const DEFAULT_LEVEL_HEMI_STRENGTH: number = 1;

export const DEFAULT_LEVEL_VIEW_OPTIONS: ILevelViewOptions = {
  ...DEFAULT_LEVEL_SURFACE_OPTIONS,
  isAntialiased: true,
  isAxesVisible: false,
  isClouded: true,
  isFogged: true,
  isGrassy: true,
  isGridVisible: false,
  isImpostors: true,
  isLamplit: true,
  isOccluded: true,
  isOcclusionCulled: true,
  isRainy: true,
  isShadowed: true,
  isSkyVisible: true,
  isWindy: true,
  isStatsVisible: true,
  isSunVisible: true,
  isWaterVisible: true,
};
