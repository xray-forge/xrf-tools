import { describe, expect, it } from "@jest/globals";
import {
  ERendererAntialiasing,
  ERendererPreset,
  IRendererFeatureSettings,
  RENDERER_FEATURE_SCHEMA,
} from "@xrf/renderer";

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
    const owned: Array<keyof IRendererFeatureSettings> = RENDER_SETTINGS_TABS.flatMap(
      (tab: IRenderSettingsTab) => tab.features
    );

    expect([...owned].sort()).toEqual(Object.keys(RENDERER_FEATURE_SCHEMA).sort());
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
      overrides: { antialiasing: ERendererAntialiasing.SMAA, grass: { isEnabled: false } },
      preset: ERendererPreset.BASE,
    };

    expect(isCustomRenderSettingsTab(getTab(ERenderSettingsTab.WORLD), choice)).toBe(true);
    expect(isCustomRenderSettingsTab(getTab(ERenderSettingsTab.IMAGE), choice)).toBe(false);
    expect(isCustomRenderSettingsTab(getTab(ERenderSettingsTab.DISPLAY), choice)).toBe(false);
  });
});
