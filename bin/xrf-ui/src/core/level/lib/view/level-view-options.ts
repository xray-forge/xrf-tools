import { DEFAULT_LEVEL_SURFACE_OPTIONS, ILevelSurfaceOptions } from "@/core/level/lib/surface/level-surface-options";

/** Everything the level viewer's toolbar switches, which is more than what a surface is drawn with. */
export interface ILevelViewOptions extends ILevelSurfaceOptions {
  /** Draws the ground plane the level sits on, in cells of a round number of metres, and the extent it claims over it. */
  isGridVisible: boolean;
  /** Draws the axis marker at the level's own origin, which is the only thing that says which way `+x` and `+z` go. */
  isAxesVisible: boolean;
  /** Marks where the level's directional light comes from with a dot in the sky; the weather's own sun is the sky's. */
  isSunMarked: boolean;
  /** Draws the two readouts over the viewport: what the frame cost, and where the camera stands. */
  isStatsVisible: boolean;
  /** Reads the light at the viewport's centre into the frame readout, in stops about middle grey. */
  isMetered: boolean;
  /**
   * Stands calibration props ahead of where the camera stood when they were turned on: a grey and a white sphere, a
   * mirror ball and a colour chart, lit as the level's surfaces are.
   */
  isCalibrationVisible: boolean;
  /** Draws the fog, which closes the level in at its distance. */
  isFogged: boolean;
  /** Draws the sky cube, rather than the backdrop behind the level. */
  isSkyVisible: boolean;
  /**
   * Fades the distance into the sky's haze, its own colour at the horizon, rather than as the engine does: towards the flat
   * fog colour, then into the sky by the fog's square, which leaves a band where the fog is darker than the horizon.
   */
  isSkyHazed: boolean;
  /** Draws the clouds over the sky. */
  isClouded: boolean;
  /** Draws the sun's lens flares over the frame, as the game's `disable_lens_flare 0` does. */
  isLensFlared: boolean;
  /** Draws the sun's light shafts through its shadow, as the game's `r2_sun_shafts` does at its highest quality. */
  isSunShafted: boolean;
  /** Rains where the weather rains. */
  isRainy: boolean;
  /** Strikes bolts where the weather strikes them. */
  isThundering: boolean;
  /** Draws a distant clump of trees as its impostor, as the game does, rather than every tree at every distance. */
  isImpostors: boolean;
  /** Casts the sun's shadows, while the settings draw them. */
  isShadowed: boolean;
  /** Darkens creases and corners by the screen's ambient occlusion, while the settings draw it. */
  isOccluded: boolean;
  /** Culls the static draws the depth hides, while the settings cull them. */
  isOcclusionCulled: boolean;
  /** Draws the opaque surfaces' depth ahead of their shading, while the settings draw it. */
  isDepthPrepassed: boolean;
  /** Smooths the frame's edges, while the settings smooth them. */
  isAntialiased: boolean;
  /** Sways the trees and the grass in the wind, as the game does. */
  isWindy: boolean;
  /** Draws the grass, while the settings draw it. */
  isGrassy: boolean;
  /** Displaces the surfaces by their height, while the settings draw parallax. */
  isParallaxed: boolean;
  /** Lights the level with its lamps, while the settings draw lights. */
  isLamplit: boolean;
  /** Draws the water, while the settings draw it. */
  isWaterVisible: boolean;
  /** Plays and draws the particle systems the level plants and its zones play. */
  isParticled: boolean;
  /** Burns the campfires, as the engine starts them, rather than leaving them to smoulder out. */
  isCampfireLit: boolean;
  /** Plays the weather's ambient effects near the camera, with the wind they bring. */
  isAmbientPlayed: boolean;
  /** Draws the spawned physics objects, breakables, boxes and vehicles. */
  isSpawnedProps: boolean;
  /** Draws the spawned items but weapons. */
  isSpawnedItems: boolean;
  /** Draws the spawned firearms and grenades. */
  isSpawnedWeapons: boolean;
  /** Draws the spawned hanging lamps' models, whatever lights them. */
  isSpawnedLamps: boolean;
  /** Draws the spawned objects a new game releases too, each with its group. */
  isSpawnedReleased: boolean;
}

/** The baked hemisphere occlusion applied whole, as the engine applies it. */
export const DEFAULT_LEVEL_HEMI_STRENGTH: number = 1;

export const DEFAULT_LEVEL_VIEW_OPTIONS: ILevelViewOptions = {
  ...DEFAULT_LEVEL_SURFACE_OPTIONS,
  isAmbientPlayed: true,
  isAntialiased: true,
  isAxesVisible: false,
  isCalibrationVisible: false,
  isCampfireLit: true,
  isClouded: true,
  isDepthPrepassed: true,
  isFogged: true,
  isGrassy: true,
  isGridVisible: false,
  isImpostors: true,
  isLamplit: true,
  isLensFlared: true,
  isMetered: false,
  isOccluded: true,
  isOcclusionCulled: true,
  isParallaxed: true,
  isParticled: true,
  isRainy: true,
  isShadowed: true,
  isSkyHazed: true,
  isSkyVisible: true,
  isSpawnedItems: true,
  isSpawnedLamps: true,
  isSpawnedProps: true,
  isSpawnedReleased: false,
  isSpawnedWeapons: true,
  isWindy: true,
  isStatsVisible: true,
  isSunMarked: false,
  isSunShafted: true,
  isThundering: true,
  isWaterVisible: true,
};
