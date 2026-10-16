import { TLevelGrassOptions } from "@/core/level/lib/features/level-feature-options";
import { TRenderFoliageSettings } from "@/core/render/lib/settings/render-feature-settings";

/** The foliage settings the Wind popover sets: how the trees and the grass move. */
export const LEVEL_FOLIAGE_MOTION_KEYS: ReadonlyArray<keyof TRenderFoliageSettings> = [
  "mode",
  "minSpeed",
  "grassSpeed",
  "grassTurbulence",
  "grassPush",
  "grassWave",
  "treesSpeed",
  "treesTrunk",
  "treesBend",
];

/** The foliage settings the Grass popover sets: the sunlight passing through leaves and grass. */
export const LEVEL_FOLIAGE_LIGHT_KEYS: ReadonlyArray<keyof TRenderFoliageSettings> = ["sssIntensity", "sssColor"];

/**
 * A view's grass with some of its foliage back at the settings: the view's foliage is one whole value, so the rest is
 * kept as the view has it, and none is kept once every key matches the settings again.
 *
 * @param grass - The view's own grass settings.
 * @param settings - The foliage the settings draw with.
 * @param keys - The foliage keys set back.
 * @returns The view's grass settings after.
 */
export function resetLevelFoliageKeys(
  grass: Partial<TLevelGrassOptions>,
  settings: TRenderFoliageSettings,
  keys: ReadonlyArray<keyof TRenderFoliageSettings>
): Partial<TLevelGrassOptions> {
  const { foliage, ...rest } = grass;

  if (!foliage) {
    return rest;
  }

  const next: TRenderFoliageSettings = { ...foliage };

  for (const key of keys) {
    Object.assign(next, { [key]: settings[key] });
  }

  const isSettings: boolean = (Object.keys(settings) as Array<keyof TRenderFoliageSettings>).every(
    (key) => next[key] === settings[key]
  );

  return isSettings ? rest : { ...rest, foliage: next };
}
