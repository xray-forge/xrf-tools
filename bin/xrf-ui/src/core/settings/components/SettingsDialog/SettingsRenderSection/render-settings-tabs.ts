import { ComponentType } from "react";

import { isRenderFeatureCustom } from "@/core/render/lib/settings/render-feature-choice";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";
import { ERenderPreset } from "@/core/render/lib/settings/render-preset";

import { SettingsRendererAmbientOcclusion } from "./SettingsRendererAmbientOcclusion";
import { SettingsRendererAntialiasing } from "./SettingsRendererAntialiasing";
import { SettingsRendererBloom } from "./SettingsRendererBloom";
import { SettingsRendererCulling } from "./SettingsRendererCulling";
import { SettingsRendererDebanding } from "./SettingsRendererDebanding";
import { SettingsRendererDisplay } from "./SettingsRendererDisplay";
import { SettingsRendererExposure } from "./SettingsRendererExposure";
import { SettingsRendererFog } from "./SettingsRendererFog";
import { SettingsRendererGlass } from "./SettingsRendererGlass";
import { SettingsRendererGrass } from "./SettingsRendererGrass";
import { SettingsRendererIndirectLight } from "./SettingsRendererIndirectLight";
import { SettingsRendererLights } from "./SettingsRendererLights";
import { SettingsRendererLod } from "./SettingsRendererLod";
import { SettingsRendererParallax } from "./SettingsRendererParallax";
import { SettingsRendererPuddles } from "./SettingsRendererPuddles";
import { SettingsRendererRainDrops } from "./SettingsRendererRainDrops";
import { SettingsRendererReflections } from "./SettingsRendererReflections";
import { SettingsRendererShadows } from "./SettingsRendererShadows";
import { SettingsRendererVolumetric } from "./SettingsRendererVolumetric";
import { SettingsRendererWater } from "./SettingsRendererWater";

/** The tabs rendering settings are split into, by what they change, as a game's graphics settings are. */
export enum ERenderSettingsTab {
  DISPLAY = "display",
  IMAGE = "image",
  LIGHTING = "lighting",
  SHADOWS = "shadows",
  WORLD = "world",
  ADVANCED = "advanced",
}

/** One tab: what it is called, which feature groups its rows set, and the blocks it draws them in. */
export interface IRenderSettingsTab {
  id: ERenderSettingsTab;
  label: string;
  /** The feature groups the tab sets, each owned by one tab alone; none where no preset sets what the tab does. */
  features: ReadonlyArray<keyof IRenderFeatureSettings>;
  /** Its blocks, top to bottom. */
  sections: ReadonlyArray<ComponentType>;
}

/** In the order the strip shows them. */
export const RENDER_SETTINGS_TABS: ReadonlyArray<IRenderSettingsTab> = [
  {
    features: [],
    id: ERenderSettingsTab.DISPLAY,
    label: "Display",
    sections: [SettingsRendererDisplay],
  },
  {
    features: [
      "antialiasing",
      "textureFiltering",
      "textureLod",
      "upscaling",
      "exposure",
      "tonemap",
      "enhancedBloom",
      "debanding",
    ],
    id: ERenderSettingsTab.IMAGE,
    label: "Image",
    sections: [
      SettingsRendererAntialiasing,
      SettingsRendererExposure,
      SettingsRendererBloom,
      SettingsRendererDebanding,
    ],
  },
  {
    features: ["lights", "volumetric", "ambientOcclusion", "indirectLight", "reflections", "rain"],
    id: ERenderSettingsTab.LIGHTING,
    label: "Lighting",
    sections: [
      SettingsRendererLights,
      SettingsRendererVolumetric,
      SettingsRendererAmbientOcclusion,
      SettingsRendererIndirectLight,
      SettingsRendererReflections,
      SettingsRendererRainDrops,
      SettingsRendererPuddles,
    ],
  },
  {
    features: ["shadows"],
    id: ERenderSettingsTab.SHADOWS,
    label: "Shadows",
    sections: [SettingsRendererShadows],
  },
  {
    features: ["water", "glass", "grass", "fog", "parallax"],
    id: ERenderSettingsTab.WORLD,
    label: "World",
    sections: [
      SettingsRendererWater,
      SettingsRendererGlass,
      SettingsRendererGrass,
      SettingsRendererFog,
      SettingsRendererParallax,
    ],
  },
  {
    features: ["lod", "isOcclusionCulled"],
    id: ERenderSettingsTab.ADVANCED,
    label: "Advanced",
    sections: [SettingsRendererLod, SettingsRendererCulling],
  },
];

/**
 * @param tab - One tab.
 * @returns Whether the preset sets what the tab does, which is when it shows the preset above its blocks.
 */
export function isPresetRenderSettingsTab(tab: IRenderSettingsTab): boolean {
  return tab.features.length > 0;
}

/**
 * @param tab - One tab.
 * @param features - Every renderer feature as the settings resolve them.
 * @param preset - The preset they stand on.
 * @returns Whether anything the tab sets differs from the preset, which its label marks.
 */
export function isCustomRenderSettingsTab(
  tab: IRenderSettingsTab,
  features: IRenderFeatureSettings,
  preset: ERenderPreset
): boolean {
  return tab.features.some((group: keyof IRenderFeatureSettings) => isRenderFeatureCustom(features, preset, group));
}
