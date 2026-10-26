import { describe, expect, it } from "@jest/globals";
import { ERenderAntialiasing } from "@xrf/ipc/types/xrf-renderer";

import { resolveRenderFeatures } from "@/core/render/lib/settings/render-feature-choice";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";
import { ERenderPreset } from "@/core/render/lib/settings/render-preset";

import {
  ERenderSettingsTab,
  IRenderSettingsTab,
  isCustomRenderSettingsTab,
  isPresetRenderSettingsTab,
  RENDER_SETTINGS_TABS,
} from "./render-settings-tabs";

function getTab(id: ERenderSettingsTab): IRenderSettingsTab {
  return RENDER_SETTINGS_TABS.find((tab: IRenderSettingsTab) => tab.id === id) as IRenderSettingsTab;
}

describe("render settings tabs", () => {
  // A feature added later has to be given a tab, or no tab would draw it nor mark it changed.
  it("gives every feature group exactly one tab", () => {
    const owned: Array<keyof IRenderFeatureSettings> = RENDER_SETTINGS_TABS.flatMap(
      (tab: IRenderSettingsTab) => tab.features
    );

    expect([...owned].sort()).toEqual(Object.keys(RENDER_FEATURE_SCHEMA).sort());
  });

  it("shows the preset over every tab but Display, which nothing of a preset's sets", () => {
    expect(
      RENDER_SETTINGS_TABS.filter((tab: IRenderSettingsTab) => !isPresetRenderSettingsTab(tab)).map(
        (tab: IRenderSettingsTab) => tab.id
      )
    ).toEqual([ERenderSettingsTab.DISPLAY]);
  });

  it("marks a tab changed where what it sets differs from the preset, and not for a value the preset has", () => {
    const choice = {
      overrides: { antialiasing: ERenderAntialiasing.TAA, grass: { isEnabled: false } },
      preset: ERenderPreset.BASE,
    };

    const features = resolveRenderFeatures(choice);

    expect(isCustomRenderSettingsTab(getTab(ERenderSettingsTab.WORLD), features, choice.preset)).toBe(true);
    expect(isCustomRenderSettingsTab(getTab(ERenderSettingsTab.IMAGE), features, choice.preset)).toBe(false);
    expect(isCustomRenderSettingsTab(getTab(ERenderSettingsTab.DISPLAY), features, choice.preset)).toBe(false);
  });
});
