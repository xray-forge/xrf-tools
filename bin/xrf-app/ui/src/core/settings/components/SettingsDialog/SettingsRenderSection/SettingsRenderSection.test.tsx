import { beforeEach, describe, expect, it } from "@jest/globals";
import { RenderResult, within } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Container } from "@wirestate/core";
import {
  ERenderDebandingMode,
  ERenderFogMode,
  ERenderIndirectLightMode,
  ERenderRainDropsMode,
  ERenderRainMode,
  ERenderReflectionMode,
  ERenderVolumetricMode,
} from "@xrf/ipc/types/xrf-renderer";

import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";
import { ERenderPreset } from "@/core/render/lib/settings/render-preset";
import { ERenderResolution } from "@/core/render/lib/settings/render-resolution";
import { RenderSettingsService } from "@/core/render/services/render-settings";
import { clearLocalStorage } from "@/core/storage";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { SettingsRenderSection } from "./SettingsRenderSection";

function renderSection(): { container: Container; settings: RenderSettingsService; view: RenderResult } {
  const container: Container = mockContainer();

  return {
    container,
    settings: container.get(RenderSettingsService),
    view: renderWithProviders(<SettingsRenderSection />, { container }),
  };
}

beforeEach(() => {
  clearLocalStorage();
});

describe("SettingsRenderSection", () => {
  it("opens on Display, which sets the frame rate with no preset over it", async () => {
    const { settings, view } = renderSection();

    expect(view.getByRole("tab", { name: "Display" })).toHaveAttribute("aria-selected", "true");
    expect(view.queryByRole("group", { name: "Preset" })).not.toBeInTheDocument();

    await userEvent.click(view.getByRole("button", { name: "30 fps" }));

    expect(settings.frameRateLimit).toBe("30");
  });

  // The output resolution sits beside the upscaling quality; only Auto has a scale to set.
  it("sets the output resolution on Image, the scale settable only while it follows the viewport", async () => {
    const { settings, view } = renderSection();

    await userEvent.click(view.getByRole("tab", { name: "Image" }));

    expect(view.getByRole("slider", { name: "Output scale" })).toBeEnabled();

    await userEvent.click(view.getByRole("button", { name: "1080p" }));

    expect(settings.renderResolution).toBe(ERenderResolution.HEIGHT_1080);
    expect(view.getByRole("slider", { name: "Output scale" })).toBeDisabled();
  });

  // Timing changes nothing drawn, so it leaves the preset as it was and marks no tab.
  it("times the passes from Display without making the settings custom", async () => {
    const { settings, view } = renderSection();

    await userEvent.click(view.getByRole("checkbox", { name: "GPU time per pass" }));

    expect(settings.isGpuTimed).toBe(true);
    expect(settings.rendererChoice).toEqual({ overrides: {}, preset: ERenderPreset.BASE });
    expect(view.queryByRole("img", { name: "changed from the preset" })).not.toBeInTheDocument();
  });

  // A change is made on top of the preset: the header says so and offers the way back, and the tab is marked.
  it("sets a feature over the preset, marks its tab, and goes back to the preset whole", async () => {
    const { settings, view } = renderSection();

    await userEvent.click(view.getByRole("tab", { name: "Image" }));
    await userEvent.click(
      within(view.getByRole("group", { name: "Antialiasing" })).getByRole("button", { name: "FXAA" })
    );

    expect(settings.rendererChoice.overrides.antialiasing).toBeDefined();
    expect(view.getByText("Custom, from Base")).toBeInTheDocument();
    expect(view.getByRole("tab", { name: "Image changed from the preset" })).toBeInTheDocument();
    expect(view.getByRole("tab", { name: "World" })).toBeInTheDocument();

    await userEvent.click(view.getByRole("button", { name: "Back to Base" }));

    expect(settings.rendererChoice).toEqual({ overrides: {}, preset: ERenderPreset.BASE });
    expect(view.getByRole("tab", { name: "Image" })).toBeInTheDocument();
  });

  it("offers the exposure's values only while the exposure adapts", async () => {
    const { settings, view } = renderSection();

    await userEvent.click(view.getByRole("tab", { name: "Image" }));

    expect(settings.rendererFeatures.exposure.isEnabled).toBe(true);
    expect(view.getByRole("slider", { name: "Middle gray" })).toBeEnabled();

    await userEvent.click(view.getByRole("checkbox", { name: "Auto exposure" }));

    expect(view.getByRole("slider", { name: "Middle gray" })).toBeDisabled();
  });

  it("turns the level's grass off for every viewport from World", async () => {
    const { settings, view } = renderSection();

    await userEvent.click(view.getByRole("tab", { name: "World" }));
    await userEvent.click(view.getByRole("checkbox", { name: "Grass" }));

    expect(settings.rendererFeatures.grass.isEnabled).toBe(false);
  });

  // The view's popovers edit these over Settings; Settings is where every viewport's defaults for them are set.
  it.each([
    [
      "Image",
      "Debanding",
      (features: IRenderFeatureSettings) => features.debanding.mode,
      ERenderDebandingMode.ENHANCED,
    ],
    [
      "Lighting",
      "Indirect light",
      (features: IRenderFeatureSettings) => features.indirectLight.mode,
      ERenderIndirectLightMode.ENHANCED,
    ],
    [
      "Lighting",
      "Screen-space reflections",
      (features: IRenderFeatureSettings) => features.reflections.mode,
      ERenderReflectionMode.ENHANCED,
    ],
    ["Lighting", "Puddles", (features: IRenderFeatureSettings) => features.rain.mode, ERenderRainMode.ENHANCED],
    [
      "Lighting",
      "Refracting drops",
      (features: IRenderFeatureSettings) => features.rain.drops,
      ERenderRainDropsMode.ENHANCED,
    ],
  ])("turns the enhanced %s %s over from its checkbox", async (tab, label, read, enhanced) => {
    const { settings, view } = renderSection();

    await userEvent.click(view.getByRole("tab", { name: tab }));

    const checkbox: HTMLElement = view.getByRole("checkbox", { name: label });
    const wasEnhanced: boolean = read(settings.rendererFeatures) === enhanced;

    expect(checkbox).toHaveProperty("checked", wasEnhanced);

    await userEvent.click(checkbox);

    expect(read(settings.rendererFeatures) === enhanced).toBe(!wasEnhanced);
  });

  it("offers the enhanced fog's strengths only while the fog is enhanced", async () => {
    const { settings, view } = renderSection();

    settings.setRendererOverrides({ fog: { mode: ERenderFogMode.ENGINE } });
    await userEvent.click(view.getByRole("tab", { name: "World" }));

    const fog: HTMLElement = view
      .getByText("Fog", { selector: "h6, span, p" })
      .closest("[data-testid=detail-section]")!;

    expect(within(fog).getByRole("slider", { name: "Height" })).toBeDisabled();

    await userEvent.click(within(fog).getByRole("button", { name: "Enhanced" }));

    expect(settings.rendererFeatures.fog.mode).toBe(ERenderFogMode.ENHANCED);
    expect(within(fog).getByRole("slider", { name: "Height" })).toBeEnabled();
  });

  it("sets the volumetric light beside the lamps on Lighting", async () => {
    const { settings, view } = renderSection();

    await userEvent.click(view.getByRole("tab", { name: "Lighting" }));

    const volumetric: HTMLElement = view
      .getByText("Volumetric light", { selector: "h6" })
      .closest("[data-testid=detail-section]")!;

    await userEvent.click(within(volumetric).getByRole("button", { name: "Engine" }));

    expect(settings.rendererFeatures.volumetric.mode).toBe(ERenderVolumetricMode.ENGINE);
    expect(within(volumetric).getByRole("slider", { name: "Distance" })).toBeDisabled();
  });
});
