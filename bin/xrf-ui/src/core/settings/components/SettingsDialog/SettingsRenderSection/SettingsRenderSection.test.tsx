import { beforeEach, describe, expect, it } from "@jest/globals";
import { RenderResult, within } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Container } from "@wirestate/core";

import { ERenderPreset } from "@/core/render/lib/settings/render-preset";
import { ERenderResolution } from "@/core/render/lib/settings/render-resolution";
import { SettingsService } from "@/core/settings/services/settings";
import { mockContainer } from "@/fixtures/utils/container";
import { renderWithProviders } from "@/fixtures/utils/render";

import { SettingsRenderSection } from "./SettingsRenderSection";

function renderSection(): { container: Container; settings: SettingsService; view: RenderResult } {
  const container: Container = mockContainer();

  return {
    container,
    settings: container.get(SettingsService),
    view: renderWithProviders(<SettingsRenderSection />, { container }),
  };
}

beforeEach(() => {
  window.localStorage.clear();
});

describe("SettingsRenderSection", () => {
  it("opens on Display, which sets the frame rate and the resolution with no preset over them", async () => {
    const { settings, view } = renderSection();

    expect(view.getByRole("tab", { name: "Display" })).toHaveAttribute("aria-selected", "true");
    expect(view.queryByRole("group", { name: "Preset" })).not.toBeInTheDocument();

    await userEvent.click(view.getByRole("button", { name: "30 fps" }));
    await userEvent.click(view.getByRole("button", { name: "1080p" }));

    expect(settings.frameRateLimit).toBe("30");
    expect(settings.renderResolution).toBe(ERenderResolution.HEIGHT_1080);
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
});
