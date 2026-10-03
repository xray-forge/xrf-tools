import { ComponentType } from "react";

import { IRenderFeatureChoice, isRenderFeatureCustom } from "@/core/render/lib/settings/render-feature-choice";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";

import { SettingsRendererAmbientOcclusion } from "./SettingsRendererAmbientOcclusion";
import { SettingsRendererAntialiasing } from "./SettingsRendererAntialiasing";
import { SettingsRendererCulling } from "./SettingsRendererCulling";
import { SettingsRendererDisplay } from "./SettingsRendererDisplay";
import { SettingsRendererExposure } from "./SettingsRendererExposure";
import { SettingsRendererGrass } from "./SettingsRendererGrass";
import { SettingsRendererLights } from "./SettingsRendererLights";
import { SettingsRendererLod } from "./SettingsRendererLod";
import { SettingsRendererShadows } from "./SettingsRendererShadows";
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
    features: ["antialiasing", "upscaling", "exposure"],
    id: ERenderSettingsTab.IMAGE,
    label: "Image",
    sections: [SettingsRendererAntialiasing, SettingsRendererExposure],
  },
  {
    features: ["lights", "ambientOcclusion"],
    id: ERenderSettingsTab.LIGHTING,
    label: "Lighting",
    sections: [SettingsRendererLights, SettingsRendererAmbientOcclusion],
  },
  {
    features: ["shadows"],
    id: ERenderSettingsTab.SHADOWS,
    label: "Shadows",
    sections: [SettingsRendererShadows],
  },
  {
    features: ["water", "grass"],
    id: ERenderSettingsTab.WORLD,
    label: "World",
    sections: [SettingsRendererWater, SettingsRendererGrass],
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
 * @param choice - The renderer's preset and what was changed on top of it.
 * @returns Whether anything the tab sets differs from the preset, which its label marks.
 */
export function isCustomRenderSettingsTab(tab: IRenderSettingsTab, choice: IRenderFeatureChoice): boolean {
  return tab.features.some((group: keyof IRenderFeatureSettings) => isRenderFeatureCustom(choice, group));
}
