import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";

import { ERenderDebandingMode, ERenderDebandingQuality } from "@/core/ipc/types/xrf-renderer";
import { LevelSkyAction } from "@/core/level/components/preview/toolbar/environment/LevelSkyAction";
import { DEFAULT_RENDER_DEBANDING_SETTINGS } from "@/core/render/lib/settings/render-feature-defaults";
import { mockLevelTextureReference } from "@/fixtures/mocks/level.mocks";
import { resetMockInvoke } from "@/fixtures/mocks/tauri.mocks";
import {
  ILevelViewContainerDescriptor,
  ILevelViewOpening,
  mockLevelViewContainer,
  renderLevelView,
  renderOpenLevelView,
  TLevelViewRenderResult,
} from "@/fixtures/utils/level-container";

/**
 * @param opening - The level and weather to open; none unless given.
 * @returns The sky's action, its popover open, and the services behind it, the sky not debanded.
 */
async function renderOpenAction(opening?: ILevelViewOpening): Promise<TLevelViewRenderResult> {
  const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

  descriptor.renderSettings.setRendererOverrides({ debanding: DEFAULT_RENDER_DEBANDING_SETTINGS });

  const rendered: TLevelViewRenderResult = opening
    ? await renderOpenLevelView(<LevelSkyAction />, descriptor, opening)
    : renderLevelView(<LevelSkyAction />, descriptor);

  await userEvent.click(rendered.getByRole("button", { name: "Sky" }));
  await rendered.findByRole("dialog", { name: "Sky" });

  return rendered;
}

beforeEach(() => {
  window.localStorage.clear();
});

afterEach(() => {
  resetMockInvoke();
});

describe("LevelSkyAction", () => {
  it("offers the skies the game's weather names, with how many keyframes use each and which are missing", async () => {
    const { getByRole, weather } = await renderOpenAction({
      weather: {
        skies: [
          { texture: mockLevelTextureReference("sky\\sky_night"), uses: 7 },
          { texture: mockLevelTextureReference("sky\\gone", false), uses: 1 },
        ],
      },
    });

    await userEvent.click(getByRole("combobox", { name: "sky_texture" }));

    expect(getByRole("listbox").textContent).toMatch(/sky\\sky_night7×.*sky\\gonemissing/);

    await userEvent.click(getByRole("option", { name: /sky_night/ }));

    expect(weather.manual?.skyTexture).toBe("sky\\sky_night");
  });

  it("takes a clouds reference typed in, and none for an empty one", async () => {
    const { getByRole, weather } = await renderOpenAction();
    const field: HTMLElement = getByRole("combobox", { name: "clouds_texture" });

    await userEvent.clear(field);
    await userEvent.type(field, "sky\\clouds_storm{Enter}");

    expect(weather.manual?.cloudsTexture).toBe("sky\\clouds_storm");
  });

  it("debands the sky on asking, offering its quality and radius only then", async () => {
    const { getByRole, queryByRole, view } = await renderOpenAction();

    expect(queryByRole("slider", { name: "Radius" })).toBeNull();

    await userEvent.click(getByRole("checkbox", { name: "Debanding" }));

    expect(view.features.debanding).toEqual({ mode: ERenderDebandingMode.ENHANCED });
    expect(getByRole("slider", { name: "Radius" })).toHaveAttribute("aria-valuetext", "48 px");

    await userEvent.click(getByRole("button", { name: "Ultra" }));

    expect(view.features.debanding).toEqual({
      mode: ERenderDebandingMode.ENHANCED,
      quality: ERenderDebandingQuality.ULTRA,
    });
  });
});
