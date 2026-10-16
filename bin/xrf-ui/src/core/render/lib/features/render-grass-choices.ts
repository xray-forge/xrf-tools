import { formatNumber } from "@xrf/format";

import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { DEFAULT_RENDER_GRASS_SETTINGS } from "@/core/render/lib/settings/render-feature-defaults";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";

/** The engine's grass density, `r__detail_density`, which is a spacing: a smaller one plants more. */
const GAME_GRASS_DENSITY: number = DEFAULT_RENDER_GRASS_SETTINGS.density;

/**
 * @param density - The engine's density, a spacing between tufts.
 * @returns How many times the game's density that is, as the settings offer it: higher is denser.
 */
export function toGrassDensityScale(density: number): number {
  return GAME_GRASS_DENSITY / density;
}

/**
 * @param scale - How many times the game's density the settings ask for.
 * @returns The engine's density that is.
 */
export function fromGrassDensityScale(scale: number): number {
  return GAME_GRASS_DENSITY / scale;
}

/**
 * The bounds each grass value is offered between, the settings' own: density as how many times the game's the tufts
 * stand, from its sparsest to three times it.
 */
export const RENDER_GRASS_LIMITS = {
  density: {
    max: toGrassDensityScale(RENDER_FEATURE_SCHEMA.grass.density.min),
    min: toGrassDensityScale(RENDER_FEATURE_SCHEMA.grass.density.max),
    step: 0.05,
  },
  height: toRenderLimits(RENDER_FEATURE_SCHEMA.grass.height, 0.1),
  radius: toRenderLimits(RENDER_FEATURE_SCHEMA.grass.radius, 1),
  shadowRadius: toRenderLimits(RENDER_FEATURE_SCHEMA.grass.shadowRadius, 1),
} as const;

/** The bounds each enhanced foliage motion value is offered between. */
export const RENDER_FOLIAGE_LIMITS = {
  grassPush: toRenderLimits(RENDER_FEATURE_SCHEMA.grass.foliage.grassPush, 0.05),
  grassSpeed: toRenderLimits(RENDER_FEATURE_SCHEMA.grass.foliage.grassSpeed, 0.1),
  grassTurbulence: toRenderLimits(RENDER_FEATURE_SCHEMA.grass.foliage.grassTurbulence, 0.05),
  grassWave: toRenderLimits(RENDER_FEATURE_SCHEMA.grass.foliage.grassWave, 0.05),
  minSpeed: toRenderLimits(RENDER_FEATURE_SCHEMA.grass.foliage.minSpeed, 0.01),
  sssColor: toRenderLimits(RENDER_FEATURE_SCHEMA.grass.foliage.sssColor, 0.05),
  sssIntensity: toRenderLimits(RENDER_FEATURE_SCHEMA.grass.foliage.sssIntensity, 0.05),
  treesBend: toRenderLimits(RENDER_FEATURE_SCHEMA.grass.foliage.treesBend, 0.05),
  treesSpeed: toRenderLimits(RENDER_FEATURE_SCHEMA.grass.foliage.treesSpeed, 0.1),
  treesTrunk: toRenderLimits(RENDER_FEATURE_SCHEMA.grass.foliage.treesTrunk, 0.01),
} as const;

/** @returns One of the enhanced foliage motion's strengths. */
export function formatFoliageStrength(strength: number): string {
  return formatNumber(strength, 2);
}

/** @returns A grass density as the settings offer it: how many times the game's. */
export function formatGrassDensity(density: number): string {
  return `${formatNumber(toGrassDensityScale(density), 2)}×`;
}

/** @returns A grass radius, in metres. */
export function formatGrassRadius(radius: number): string {
  return `${formatNumber(radius, 0)} m`;
}

/** @returns A grass height, as the multiple of the game's it is. */
export function formatGrassHeight(height: number): string {
  return `${formatNumber(height, 1)}×`;
}
