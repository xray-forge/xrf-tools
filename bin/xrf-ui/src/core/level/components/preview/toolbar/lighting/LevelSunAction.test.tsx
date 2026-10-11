import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";
import { act } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";

import { ERenderSunShaftsQuality, ERenderVolumetricMode } from "@/core/ipc/types/xrf-renderer";
import { LevelSunAction } from "@/core/level/components/preview/toolbar/lighting/LevelSunAction";
import { DEFAULT_LEVEL_VIEW_OPTIONS } from "@/core/level/lib/view/level-view-options";
import { DEFAULT_LEVEL_MANUAL_WEATHER } from "@/core/level/lib/weather/level-manual-weather";
import { DEFAULT_RENDER_VOLUMETRIC_SETTINGS } from "@/core/render/lib/settings/render-feature-defaults";
import { clearLocalStorage } from "@/core/storage";
import { resetMockInvoke } from "@/fixtures/mocks/tauri.mocks";
import {
  ILevelViewContainerDescriptor,
  ILevelViewOpening,
  mockLevelViewContainer,
  noteLevelApplied,
  renderLevelView,
  renderOpenLevelView,
  TLevelViewRenderResult,
} from "@/fixtures/utils/level-container";

/**
 * @param opening - The level and weather to open; none unless given.
 * @returns The sun's action and the services behind it, no volumetric lighting drawn.
 */
async function renderAction(opening?: ILevelViewOpening): Promise<TLevelViewRenderResult> {
  const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

  descriptor.renderSettings.setRendererOverrides({ volumetric: DEFAULT_RENDER_VOLUMETRIC_SETTINGS });

  return opening
    ? renderOpenLevelView(<LevelSunAction />, descriptor, opening)
    : renderLevelView(<LevelSunAction />, descriptor);
}

/** Opens the sun's popover: no toggle sits on it, so a click does. */
async function openSettings(rendered: TLevelViewRenderResult): Promise<void> {
  await userEvent.click(rendered.getByRole("button", { name: "Sun" }));
  await rendered.findByRole("dialog", { name: "Sun" });
}

beforeEach(() => {
  clearLocalStorage();
});

afterEach(() => {
  resetMockInvoke();
});

describe("LevelSunAction", () => {
  it("stands the sun where the level was compiled against, and sets its keys back to default_clear's noon", async () => {
    const rendered = await renderAction({ level: { sun: { color: [1, 1, 1], direction: { x: 0, y: -1, z: 1 } } } });

    act(() => rendered.weather.editManual({ sunAltitude: 10, sunLongitude: -80 }));
    await openSettings(rendered);

    expect(rendered.getByRole("dialog", { name: "Sun" }).textContent).toContain("80° up at a bearing of 10°");

    await userEvent.click(rendered.getByRole("button", { name: "Use the level's compiled sun" }));

    expect(rendered.weather.manual?.sunAltitude).toBeCloseTo(0);
    expect(rendered.weather.manual?.sunLongitude).toBeCloseTo(-45);

    await userEvent.click(rendered.getByRole("button", { name: "Back to default_clear's noon" }));

    expect(rendered.weather.manual).toMatchObject({
      ambientColor: DEFAULT_LEVEL_MANUAL_WEATHER.ambientColor,
      hemisphereColor: DEFAULT_LEVEL_MANUAL_WEATHER.hemisphereColor,
      sun: DEFAULT_LEVEL_MANUAL_WEATHER.sun,
      sunAltitude: DEFAULT_LEVEL_MANUAL_WEATHER.sunAltitude,
      sunColor: DEFAULT_LEVEL_MANUAL_WEATHER.sunColor,
      sunLongitude: DEFAULT_LEVEL_MANUAL_WEATHER.sunLongitude,
      sunShaftsIntensity: DEFAULT_LEVEL_MANUAL_WEATHER.sunShaftsIntensity,
    });
  });

  // The sky draws the weather's sun or moon as the game does; the button says whether one shows now.
  it("says whether the sky draws a sun or a moon, naming it", async () => {
    const { getByRole, render } = await renderAction();

    noteLevelApplied(render, { sun: "moon_halo_full" });

    expect(getByRole("button", { name: "Sun" })).toHaveAccessibleDescription(/moon_halo_full in the sky$/);

    noteLevelApplied(render, { sun: null });

    expect(getByRole("button", { name: "Sun" })).toHaveAccessibleDescription(/none in the sky$/);
  });

  // The lens flare and the shafts are the sun's: their switches and the keys a keyframe draws them by sit together.
  it("switches the lens flares and the sunshafts, and edits the sun and the shafts' density", async () => {
    const rendered = await renderAction({ weather: { suns: ["gradient1", "moon_halo_full"] } });

    await openSettings(rendered);
    await userEvent.click(rendered.getByRole("checkbox", { name: "Lens flares" }));
    await userEvent.click(rendered.getByRole("checkbox", { name: "Sunshafts" }));

    expect(rendered.view.options.isLensFlared).toBe(!DEFAULT_LEVEL_VIEW_OPTIONS.isLensFlared);
    expect(rendered.view.options.isSunShafted).toBe(!DEFAULT_LEVEL_VIEW_OPTIONS.isSunShafted);

    await userEvent.click(rendered.getByRole("combobox", { name: "sun" }));
    await userEvent.click(await rendered.findByRole("option", { name: "moon_halo_full" }));

    expect(rendered.weather.manual?.sun).toBe("moon_halo_full");
  });

  it("sets the level's sun shafts' quality and the floor under their density", async () => {
    const rendered = await renderAction();

    await openSettings(rendered);
    await userEvent.click(rendered.getByRole("button", { name: "Low" }));

    expect(rendered.weather.sunShafts).toEqual({ minimum: 0, quality: ERenderSunShaftsQuality.LOW });
    expect(rendered.getByRole("slider", { name: "Minimum" })).toHaveAttribute("aria-valuemax", "0.5");
  });

  // The froxel volume replaces the sun's shafts while enhanced, so its settings sit beside them.
  it("sets the view's volumetric lighting from the sun's settings", async () => {
    const rendered = await renderAction();

    await openSettings(rendered);
    await userEvent.click(rendered.getByRole("button", { name: "Enhanced" }));

    expect(rendered.view.features.volumetric).toEqual({ mode: ERenderVolumetricMode.ENHANCED });
  });
});
