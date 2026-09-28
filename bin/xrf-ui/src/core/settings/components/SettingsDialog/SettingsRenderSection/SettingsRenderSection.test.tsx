import { beforeEach, describe, expect, it } from "@jest/globals";
import { RenderResult, within } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { Container } from "@wirestate/core";
import { ERendererPreset, ERenderResolution } from "@xrf/renderer";

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
  it("sets the frame rate every viewport draws at, and the resolution it draws in", async () => {
    const { settings, view } = renderSection();

    await userEvent.click(view.getByRole("button", { name: "30 fps" }));
    await userEvent.click(view.getByRole("button", { name: "1080p" }));

    expect(settings.frameRateLimit).toBe("30");
    expect(settings.renderResolution).toBe(ERenderResolution.HEIGHT_1080);
  });

  // A change is made on top of the preset, and the section says so and offers the way back.
  it("sets a feature over the preset, and goes back to the preset whole", async () => {
    const { settings, view } = renderSection();

    await userEvent.click(
      within(view.getByRole("group", { name: "Antialiasing" })).getByRole("button", { name: "FXAA" })
    );

    expect(settings.rendererChoice.overrides.antialiasing).toBeDefined();
    expect(view.getByText("Custom, from Base")).toBeInTheDocument();

    await userEvent.click(view.getByRole("button", { name: "Back to Base" }));

    expect(settings.rendererChoice).toEqual({ overrides: {}, preset: ERendererPreset.BASE });
  });

  it("offers the exposure's values only while the exposure adapts", async () => {
    const { settings, view } = renderSection();

    expect(settings.rendererFeatures.exposure.isEnabled).toBe(true);
    expect(view.getByRole("slider", { name: "Middle gray" })).toBeEnabled();

    await userEvent.click(view.getByRole("checkbox", { name: "Exposure adaptation" }));

    expect(view.getByRole("slider", { name: "Middle gray" })).toBeDisabled();
  });

  it("turns the level's grass off for every viewport", async () => {
    const { settings, view } = renderSection();

    await userEvent.click(view.getByRole("checkbox", { name: "Grass" }));

    expect(settings.rendererFeatures.grass.isEnabled).toBe(false);
  });
});
