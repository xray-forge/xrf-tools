import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";
import { userEvent } from "@testing-library/user-event";
import { resetMockInvoke } from "@xrf/ipc/testing";

import { LevelSkyAction } from "@/applications/level-viewer/components/toolbar/environment/LevelSkyAction";
import {
  ILevelViewContainerDescriptor,
  ILevelViewOpening,
  mockLevelViewContainer,
  renderLevelView,
  renderOpenLevelView,
  TLevelViewRenderResult,
} from "@/core/level/fixtures/level-container";
import { mockLevelTextureReference } from "@/core/level/fixtures/level.mocks";
import { clearLocalStorage } from "@/core/storage";

/**
 * @param opening - The level and weather to open; none unless given.
 * @returns The sky's action, its popover open, and the services behind it.
 */
async function renderOpenAction(opening?: ILevelViewOpening): Promise<TLevelViewRenderResult> {
  const descriptor: ILevelViewContainerDescriptor = mockLevelViewContainer();

  const rendered: TLevelViewRenderResult = opening
    ? await renderOpenLevelView(<LevelSkyAction />, descriptor, opening)
    : renderLevelView(<LevelSkyAction />, descriptor);

  await userEvent.click(rendered.getByRole("button", { name: "Sky" }));
  await rendered.findByRole("dialog", { name: "Sky" });

  return rendered;
}

beforeEach(() => {
  clearLocalStorage();
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

    await userEvent.click(getByRole("combobox", { name: "Sky texture" }));

    expect(getByRole("listbox").textContent).toMatch(/sky\\sky_night7×.*sky\\gonemissing/);

    await userEvent.click(getByRole("option", { name: /sky_night/ }));

    expect(weather.manual?.skyTexture).toBe("sky\\sky_night");
  });

  it("takes a clouds reference typed in, and none for an empty one", async () => {
    const { getByRole, weather } = await renderOpenAction();
    const field: HTMLElement = getByRole("combobox", { name: "Cloud texture" });

    await userEvent.clear(field);
    await userEvent.type(field, "sky\\clouds_storm{Enter}");

    expect(weather.manual?.cloudsTexture).toBe("sky\\clouds_storm");
  });
});
