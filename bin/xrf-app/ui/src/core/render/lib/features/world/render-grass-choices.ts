import { formatDegrees, formatNumber, formatPercent } from "@xrf/format";
import { ERenderFoliageMode, RenderFoliageMode } from "@xrf/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";

import { IRenderLimits, toRenderLimits } from "@/core/render/lib/features/render-limits";
import {
  ERenderSettingControl,
  IRenderSliderSetting,
  TRenderSettings,
} from "@/core/render/lib/features/render-setting";
import { DEFAULT_RENDER_GRASS_SETTINGS } from "@/core/render/lib/settings/choice/render-feature-defaults";
import {
  TRenderFoliageSettings,
  TRenderGrassSettings,
} from "@/core/render/lib/settings/choice/render-feature-settings";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/schema/render-feature-schema";

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
 * The bounds the grass's density is offered between, the settings' own: how many times the game's the tufts stand,
 * from its sparsest to three times it.
 */
export const RENDER_GRASS_DENSITY_LIMITS: IRenderLimits = {
  max: toGrassDensityScale(RENDER_FEATURE_SCHEMA.grass.density.min),
  min: toGrassDensityScale(RENDER_FEATURE_SCHEMA.grass.density.max),
  step: 0.05,
};

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

/** @returns How many of the sun's nearest cascades the grass's cards are drawn into, none for the contact shadows. */
export function formatGrassShadowCascades(cascades: number): string {
  if (cascades <= 0) {
    return "contact only";
  }

  return cascades === 1 ? "1 cascade" : `${formatNumber(cascades, 0)} cascades`;
}

/** @returns The slope past which no grass grows, in degrees. */
export function formatGrassSlopeLimit(slope: number): string {
  return formatDegrees(slope);
}

/** @returns A grass height, as the multiple of the game's it is. */
export function formatGrassHeight(height: number): string {
  return `${formatNumber(height, 1)}×`;
}

const RENDER_FOLIAGE_MODE_DESCRIPTIONS: Readonly<Record<RenderFoliageMode, string>> = {
  [ERenderFoliageMode.ENGINE]:
    "Leans each tree by the weather's tree wind and waves each tuft on its own, as the game does.",
  [ERenderFoliageMode.ENHANCED]:
    "Swings trunks downwind and moves branches and grass on a flow field drifting with the wind.",
};

/**
 * @param mode - A foliage motion mode.
 * @returns What it does, in a sentence.
 */
export function explainRenderFoliageMode(mode: RenderFoliageMode): string {
  return RENDER_FOLIAGE_MODE_DESCRIPTIONS[mode];
}

/** The foliage motion modes, in the order they are offered. */
export const RENDER_FOLIAGE_MODE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderFoliageMode>> = [
  { label: "Engine", value: ERenderFoliageMode.ENGINE },
  { label: "Enhanced", value: ERenderFoliageMode.ENHANCED },
];

/** Whether the grass casts no shadows, which none of its shadows' values tune. */
function isUnshadowedGrass(grass: TRenderGrassSettings): boolean {
  return !grass.isShadowed;
}

/**
 * How each grass setting is set, in the settings and the Grass popover alike. The density is offered as how many
 * times the game's the tufts stand, a control of its own over `RENDER_GRASS_DENSITY_LIMITS`.
 */
export const RENDER_GRASS_SETTINGS: TRenderSettings<TRenderGrassSettings> = {
  density: {
    control: ERenderSettingControl.CUSTOM,
    description: "Multiplier over the game's density. Higher costs more to generate and draw.",
    label: "Density",
  },
  foliage: { control: ERenderSettingControl.CUSTOM, label: "Foliage" },
  height: {
    control: ERenderSettingControl.SLIDER,
    description: "Height multiplier.",
    format: formatGrassHeight,
    label: "Height",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.grass.height, 0.1),
  },
  isAligned: {
    control: ERenderSettingControl.CHECKBOX,
    description: "Stands each tuft along its ground's slope rather than straight up.",
    label: "Aligned to the ground",
  },
  isEnabled: {
    control: ERenderSettingControl.CHECKBOX,
    description: "Off, terrain is drawn bare.",
    label: "Grass",
  },
  isShadowed: {
    control: ERenderSettingControl.CHECKBOX,
    description:
      "Casts shadows from the grass: by the contact shadows, and as cards traced under ray reconstruction or in the " +
      "sun's nearest cascades.",
    label: "Shadows",
  },
  radius: {
    control: ERenderSettingControl.SLIDER,
    description: "Draw distance in metres, fading at the edge.",
    format: formatGrassRadius,
    label: "Radius",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.grass.radius, 1),
  },
  shadowCascades: {
    control: ERenderSettingControl.SLIDER,
    description:
      "How many of the sun's nearest cascades draw the grass's own shadows, as far as the radius; the contact " +
      "shadows shade the rest.",
    format: formatGrassShadowCascades,
    isInert: isUnshadowedGrass,
    label: "Shadow cascades",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.grass.shadowCascades, 1),
  },
  shadowRadius: {
    control: ERenderSettingControl.SLIDER,
    description: "How far from the camera the grass casts shadows, fading out over the last fifth.",
    format: formatGrassRadius,
    isInert: isUnshadowedGrass,
    label: "Shadow radius",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.grass.shadowRadius, 1),
  },
  slopeLimit: {
    control: ERenderSettingControl.SLIDER,
    description: "Degrees of slope past which no grass grows, thinning out over the last fifth; 90 for none.",
    format: formatGrassSlopeLimit,
    label: "Slope limit",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.grass.slopeLimit, 1),
  },
};

/**
 * @param label - What it is called.
 * @param limits - Between what it is offered.
 * @param format - How it reads.
 * @returns One of the enhanced foliage's values, which tune nothing of the game's motion.
 */
function toFoliageSlider(
  label: string,
  limits: IRenderLimits,
  format: (value: number) => string = formatFoliageStrength
): IRenderSliderSetting<TRenderFoliageSettings> {
  return {
    control: ERenderSettingControl.SLIDER,
    format,
    isInert: (foliage: TRenderFoliageSettings) => foliage.mode !== ERenderFoliageMode.ENHANCED,
    label,
    limits,
  };
}

/** How each foliage setting is set: its motion in the Wind popover, the light through its leaves in the Grass one. */
export const RENDER_FOLIAGE_SETTINGS: TRenderSettings<TRenderFoliageSettings> = {
  grassPush: toFoliageSlider("Grass push", toRenderLimits(RENDER_FEATURE_SCHEMA.grass.foliage.grassPush, 0.05)),
  grassSpeed: toFoliageSlider("Grass speed", toRenderLimits(RENDER_FEATURE_SCHEMA.grass.foliage.grassSpeed, 0.1)),
  grassTurbulence: toFoliageSlider(
    "Grass turbulence",
    toRenderLimits(RENDER_FEATURE_SCHEMA.grass.foliage.grassTurbulence, 0.05)
  ),
  grassWave: toFoliageSlider("Grass wave", toRenderLimits(RENDER_FEATURE_SCHEMA.grass.foliage.grassWave, 0.05)),
  minSpeed: toFoliageSlider(
    "Least wind",
    toRenderLimits(RENDER_FEATURE_SCHEMA.grass.foliage.minSpeed, 0.01),
    formatPercent
  ),
  mode: {
    control: ERenderSettingControl.CHOICE,
    explain: explainRenderFoliageMode,
    isEntryDescribed: true,
    label: "Mode",
    options: RENDER_FOLIAGE_MODE_OPTIONS,
  },
  sssColor: toFoliageSlider(
    "Sun colour through leaves",
    toRenderLimits(RENDER_FEATURE_SCHEMA.grass.foliage.sssColor, 0.05)
  ),
  sssIntensity: toFoliageSlider(
    "Light through leaves",
    toRenderLimits(RENDER_FEATURE_SCHEMA.grass.foliage.sssIntensity, 0.05)
  ),
  treesBend: toFoliageSlider("Trunk bend", toRenderLimits(RENDER_FEATURE_SCHEMA.grass.foliage.treesBend, 0.05)),
  treesSpeed: toFoliageSlider("Branches speed", toRenderLimits(RENDER_FEATURE_SCHEMA.grass.foliage.treesSpeed, 0.1)),
  treesTrunk: toFoliageSlider("Trunk speed", toRenderLimits(RENDER_FEATURE_SCHEMA.grass.foliage.treesTrunk, 0.01)),
};
